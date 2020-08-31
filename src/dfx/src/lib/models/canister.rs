use crate::config::cache::Cache;
use crate::lib::builders::{
    BuildConfig, BuildOutput, BuilderPool, CanisterBuilder, IdlBuildOutput, WasmBuildOutput,
};
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{BuildErrorKind, DfxError, DfxResult};
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::util::assets;
use ic_types::principal::Principal as CanisterId;
use petgraph::graph::{DiGraph, NodeIndex};
use rand::{thread_rng, RngCore};
use serde::Deserialize;
use slog::Logger;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

/// Represents a canister from a DFX project. It can be a virtual Canister.
/// Multiple canister instances can have the same info, but would be differentiated
/// by their IDs.
/// Once an instance of a canister is built it is immutable. So for comparing
/// two canisters one can use their ID.
pub struct Canister {
    info: CanisterInfo,
    builder: Arc<dyn CanisterBuilder>,
    output: RefCell<Option<BuildOutput>>,
}

impl Canister {
    /// Create a new canister.
    /// This can only be done by a CanisterPool.
    pub(super) fn new(info: CanisterInfo, builder: Arc<dyn CanisterBuilder>) -> Self {
        Self {
            info,
            builder,
            output: RefCell::new(None),
        }
    }

    pub fn prebuild(&self, pool: &CanisterPool, build_config: &BuildConfig) -> DfxResult {
        self.builder.prebuild(pool, &self.info, build_config)
    }

    pub fn build(
        &self,
        pool: &CanisterPool,
        build_config: &BuildConfig,
    ) -> DfxResult<&BuildOutput> {
        let output = self.builder.build(pool, &self.info, build_config)?;

        // Ignore the old output, and return a reference.
        let _ = self.output.replace(Some(output));
        Ok(self.get_build_output().unwrap())
    }

    pub fn postbuild(&self, pool: &CanisterPool, build_config: &BuildConfig) -> DfxResult {
        self.builder.postbuild(pool, &self.info, build_config)
    }

    pub fn get_name(&self) -> &str {
        self.info.get_name()
    }

    pub fn canister_id(&self) -> CanisterId {
        self.info.get_canister_id().unwrap()
    }

    // this function is only ever used when build_config.build_mode_check is true
    pub fn generate_random_canister_id() -> DfxResult<CanisterId> {
        let mut rng = thread_rng();
        let mut v: Vec<u8> = std::iter::repeat(0u8).take(8).collect();
        rng.fill_bytes(v.as_mut_slice());
        Ok(CanisterId::try_from(v)?)
    }

    /// Get the build output of a build process. If the output isn't known at this time,
    /// will return [None].
    pub fn get_build_output(&self) -> Option<&BuildOutput> {
        unsafe { (&*self.output.as_ptr()).as_ref() }
    }
}

/// A canister pool is a list of canisters.
pub struct CanisterPool {
    canisters: Vec<Arc<Canister>>,
    logger: Logger,
    cache: Arc<dyn Cache>,
}

impl CanisterPool {
    fn insert(
        env: &dyn Environment,
        generate_cid: bool,
        canister_name: &str,
        canisters_map: &mut Vec<Arc<Canister>>,
    ) -> DfxResult<()> {
        let builder_pool = BuilderPool::new(env)?;
        let canister_id_store = CanisterIdStore::for_env(env)?;
        let config = env
            .get_config()
            .ok_or(DfxError::CommandMustBeRunInAProject)?;

        let canister_id = match canister_id_store.find(canister_name) {
            Some(canister_id) => Some(canister_id),
            None if generate_cid => Some(Canister::generate_random_canister_id()?),
            _ => None,
        };
        let info = CanisterInfo::load(&config, canister_name, canister_id)?;

        if let Some(builder) = builder_pool.get(&info) {
            canisters_map.insert(0, Arc::new(Canister::new(info, builder)));
            Ok(())
        } else {
            Err(DfxError::CouldNotFindBuilderForCanister(
                info.get_name().to_string(),
            ))
        }
    }

    pub fn load(
        env: &dyn Environment,
        generate_cid: bool,
        some_canister: Option<&str>,
    ) -> DfxResult<Self> {
        let logger = env.get_logger().new(slog::o!());
        let config = env
            .get_config()
            .ok_or(DfxError::CommandMustBeRunInAProject)?;

        let mut canisters_map = Vec::new();

        if let Some(canister_name) = some_canister {
            CanisterPool::insert(env, generate_cid, canister_name, &mut canisters_map)?;

            // get direct dependencies
            let canister_id = CanisterIdStore::for_env(env)?.get(canister_name)?;
            let info = CanisterInfo::load(&config, canister_name, Some(canister_id))?;
            let deps = match info.get_extra_value("dependencies") {
                None => vec![],
                Some(v) => Vec::<String>::deserialize(v).map_err(|_| {
                    DfxError::Unknown(String::from("Field 'dependencies' is of the wrong type"))
                })?,
            };

            for canister in deps.iter() {
                CanisterPool::insert(env, generate_cid, canister, &mut canisters_map)?;
            }
        } else {
            // insert all canisters configured in dfx.json
            let canisters = config.get_config().canisters.as_ref().ok_or_else(|| {
                DfxError::Unknown("No canisters in the configuration file.".to_string())
            })?;
            for (key, _value) in canisters.iter() {
                CanisterPool::insert(env, generate_cid, key, &mut canisters_map)?;
            }
        }

        Ok(CanisterPool {
            canisters: canisters_map,
            logger,
            cache: env.get_cache().clone(),
        })
    }

    pub fn get_canister(&self, canister_id: &CanisterId) -> Option<&Canister> {
        for c in &self.canisters {
            let info = &c.info;
            if Some(canister_id) == info.get_canister_id().ok().as_ref() {
                return Some(c);
            }
        }
        None
    }

    pub fn get_canister_list(&self) -> Vec<&Canister> {
        self.canisters.iter().map(|c| c.as_ref()).collect()
    }

    pub fn get_canister_info(&self, canister_id: &CanisterId) -> Option<&CanisterInfo> {
        self.get_canister(canister_id).map(|c| &c.info)
    }

    pub fn get_first_canister_with_name(&self, name: &str) -> Option<Arc<Canister>> {
        for c in &self.canisters {
            if c.info.get_name() == name {
                return Some(Arc::clone(c));
            }
        }
        None
    }

    pub fn get_logger(&self) -> &Logger {
        &self.logger
    }

    fn build_dependencies_graph(&self) -> DfxResult<DiGraph<CanisterId, ()>> {
        let mut graph: DiGraph<CanisterId, ()> = DiGraph::new();
        let mut id_set: BTreeMap<CanisterId, NodeIndex<u32>> = BTreeMap::new();

        // Add all the canisters as nodes.
        for canister in &self.canisters {
            let canister_id = canister.info.get_canister_id()?;
            id_set.insert(canister_id.clone(), graph.add_node(canister_id.clone()));
        }

        // Add all the edges.
        for canister in &self.canisters {
            let canister_id = canister.canister_id();
            let canister_info = &canister.info;
            let deps = canister.builder.get_dependencies(self, canister_info)?;
            if let Some(node_ix) = id_set.get(&canister_id) {
                for d in deps {
                    if let Some(dep_ix) = id_set.get(&d) {
                        graph.add_edge(*node_ix, *dep_ix, ());
                    }
                }
            }
        }

        // Verify the graph has no cycles.
        if let Err(err) = petgraph::algo::toposort(&graph, None) {
            match graph.node_weight(err.node_id()) {
                Some(canister_id) => Err(DfxError::BuildError(BuildErrorKind::CircularDependency(
                    match self.get_canister_info(canister_id) {
                        Some(info) => info.get_name().to_string(),
                        None => format!("<{}>", canister_id.to_text()),
                    },
                ))),
                None => Err(DfxError::BuildError(BuildErrorKind::CircularDependency(
                    "<Unknown>".to_string(),
                ))),
            }
        } else {
            Ok(graph)
        }
    }

    fn step_prebuild_all(&self, _build_config: &BuildConfig) -> DfxResult<()> {
        Ok(())
    }

    fn step_prebuild(&self, build_config: &BuildConfig, canister: &Canister) -> DfxResult<()> {
        canister.prebuild(self, build_config)
    }

    fn step_build<'a>(
        &self,
        build_config: &BuildConfig,
        canister: &'a Canister,
    ) -> DfxResult<&'a BuildOutput> {
        canister.build(self, build_config)
    }

    fn step_postbuild(
        &self,
        build_config: &BuildConfig,
        canister: &Canister,
        build_output: &BuildOutput,
    ) -> DfxResult<()> {
        // Copy the WASM and IDL files to canisters/NAME/...
        let IdlBuildOutput::File(build_idl_path) = &build_output.idl;
        let idl_file_path = canister.info.get_build_idl_path();
        if build_idl_path.ne(&idl_file_path) {
            std::fs::create_dir_all(idl_file_path.parent().unwrap())?;
            std::fs::copy(&build_idl_path, &idl_file_path)
                .map(|_| {})
                .map_err(DfxError::from)?;

            let mut perms = std::fs::metadata(&idl_file_path)?.permissions();
            perms.set_readonly(false);
            std::fs::set_permissions(&idl_file_path, perms)?;
        }

        let WasmBuildOutput::File(build_wasm_path) = &build_output.wasm;
        let wasm_file_path = canister.info.get_build_wasm_path();
        if build_wasm_path.ne(&wasm_file_path) {
            std::fs::create_dir_all(wasm_file_path.parent().unwrap())?;
            std::fs::copy(&build_wasm_path, &wasm_file_path)
                .map(|_| {})
                .map_err(DfxError::from)?;

            let mut perms = std::fs::metadata(&wasm_file_path)?.permissions();
            perms.set_readonly(false);
            std::fs::set_permissions(&wasm_file_path, perms)?;
        }

        // And then create an canisters/IDL folder with canister DID files per canister ID.
        let idl_root = &build_config.idl_root;
        let canister_id = canister.canister_id();
        let idl_file_path = idl_root.join(canister_id.to_text()).with_extension("did");

        std::fs::create_dir_all(idl_file_path.parent().unwrap())?;
        std::fs::copy(&build_idl_path, &idl_file_path)
            .map(|_| {})
            .map_err(DfxError::from)?;

        build_canister_js(self.cache.clone(), &canister.canister_id(), &canister.info)?;

        canister.postbuild(self, build_config)
    }

    fn step_postbuild_all(
        &self,
        build_config: &BuildConfig,
        _order: &[CanisterId],
    ) -> DfxResult<()> {
        // We don't want to simply remove the whole directory, as in the future,
        // we may want to keep the IDL files downloaded from network.
        for canister in &self.canisters {
            let idl_root = &build_config.idl_root;
            let canister_id = canister.canister_id();
            let idl_file_path = idl_root.join(canister_id.to_text()).with_extension("did");

            // Ignore errors (e.g. File Not Found).
            let _ = std::fs::remove_file(idl_file_path);
        }

        Ok(())
    }

    /// Build all canisters, returning a vector of results of each builds.
    pub fn build(
        &self,
        build_config: BuildConfig,
    ) -> DfxResult<Vec<Result<&BuildOutput, BuildErrorKind>>> {
        self.step_prebuild_all(&build_config).map_err(|e| {
            DfxError::BuildError(BuildErrorKind::PrebuildAllStepFailed(Box::new(e)))
        })?;

        let graph = self.build_dependencies_graph()?;
        let order: Vec<CanisterId> = petgraph::algo::toposort(&graph, None)
            .map_err(|cycle| match graph.node_weight(cycle.node_id()) {
                Some(canister_id) => DfxError::BuildError(BuildErrorKind::CircularDependency(
                    match self.get_canister_info(canister_id) {
                        Some(info) => info.get_name().to_string(),
                        None => format!("<{}>", canister_id.to_text()),
                    },
                )),
                None => DfxError::BuildError(BuildErrorKind::CircularDependency(
                    "<Unknown>".to_string(),
                )),
            })?
            .iter()
            .rev() // Reverse the order, as we have a dependency graph, we want to reverse indices.
            .map(|idx| graph.node_weight(*idx).unwrap().clone())
            .collect();

        let mut result = Vec::new();
        for canister_id in &order {
            if let Some(canister) = self.get_canister(canister_id) {
                result.push(
                    self.step_prebuild(&build_config, canister)
                        .map_err(|e| {
                            BuildErrorKind::PrebuildStepFailed(canister_id.clone(), Box::new(e))
                        })
                        .and_then(|_| {
                            self.step_build(&build_config, canister).map_err(|e| {
                                BuildErrorKind::BuildStepFailed(canister_id.clone(), Box::new(e))
                            })
                        })
                        .and_then(|o| {
                            self.step_postbuild(&build_config, canister, o)
                                .map_err(|e| {
                                    BuildErrorKind::PostbuildStepFailed(
                                        canister_id.clone(),
                                        Box::new(e),
                                    )
                                })
                                .map(|_| o)
                        }),
                );
            }
        }

        self.step_postbuild_all(&build_config, &order)
            .map_err(|e| {
                DfxError::BuildError(BuildErrorKind::PostbuildAllStepFailed(Box::new(e)))
            })?;

        Ok(result)
    }

    /// Build all canisters, failing with the first that failed the build. Will return
    /// nothing if all succeeded.
    pub fn build_or_fail(&self, build_config: BuildConfig) -> DfxResult<()> {
        let outputs = self.build(build_config)?;

        for output in outputs {
            output.map_err(DfxError::BuildError)?;
        }

        Ok(())
    }
}

fn decode_path_to_str(path: &Path) -> DfxResult<&str> {
    path.to_str().ok_or_else(|| {
        DfxError::BuildError(BuildErrorKind::CanisterJsGenerationError(format!(
            "Unable to convert output canister js path to a string: {:#?}",
            path
        )))
    })
}

/// Create a canister JavaScript DID and Actor Factory.
fn build_canister_js(
    cache: Arc<dyn Cache>,
    canister_id: &CanisterId,
    canister_info: &CanisterInfo,
) -> DfxResult {
    let output_did_js_path = canister_info.get_build_idl_path().with_extension("did.js");
    let output_canister_js_path = canister_info.get_build_idl_path().with_extension("js");

    let mut cmd = cache.get_binary_command("didc")?;
    let cmd = cmd
        .arg("--js")
        .arg(&canister_info.get_build_idl_path())
        .arg("-o")
        .arg(&output_did_js_path);

    let output = cmd.output()?;
    if !output.status.success() {
        return Err(DfxError::BuildError(BuildErrorKind::CompilerError(
            format!("{:?}", cmd),
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        )));
    } else if !output.stderr.is_empty() {
        // Cannot use eprintln, because it would interfere with the progress bar.
        println!("{}", String::from_utf8_lossy(&output.stderr));
    }

    let mut language_bindings = assets::language_bindings()?;

    for f in language_bindings.entries()? {
        let mut file = f?;
        let mut file_contents = String::new();
        file.read_to_string(&mut file_contents)?;

        let new_file_contents = file_contents
            .replace("{canister_id}", &canister_id.to_text())
            .replace("{project_name}", canister_info.get_name());

        match decode_path_to_str(&file.path()?)? {
            "canister.js" => {
                std::fs::write(
                    decode_path_to_str(&output_canister_js_path)?,
                    new_file_contents,
                )?;
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}
