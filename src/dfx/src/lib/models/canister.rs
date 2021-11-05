use crate::config::dfinity::Config;
use crate::lib::builders::{
    BuildConfig, BuildOutput, BuilderPool, CanisterBuilder, IdlBuildOutput, WasmBuildOutput,
};
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{BuildError, DfxError, DfxResult};
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::util::{assets, check_candid_file};

use anyhow::anyhow;
use ic_types::principal::Principal as CanisterId;
use petgraph::graph::{DiGraph, NodeIndex};
use rand::{thread_rng, RngCore};
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

    pub fn generate(&self, pool: &CanisterPool, build_config: &BuildConfig) -> DfxResult {
        self.builder.generate(pool, &self.info, build_config)
    }
}

/// A canister pool is a list of canisters.
pub struct CanisterPool {
    canisters: Vec<Arc<Canister>>,
    logger: Logger,
}

struct PoolConstructHelper<'a> {
    config: &'a Config,
    builder_pool: BuilderPool,
    canister_id_store: CanisterIdStore,
    generate_cid: bool,
    canisters_map: &'a mut Vec<Arc<Canister>>,
}

impl CanisterPool {
    fn insert(canister_name: &str, pool_helper: &mut PoolConstructHelper<'_>) -> DfxResult<()> {
        let canister_id = match pool_helper.canister_id_store.find(canister_name) {
            Some(canister_id) => Some(canister_id),
            None if pool_helper.generate_cid => Some(Canister::generate_random_canister_id()?),
            _ => None,
        };
        let info = CanisterInfo::load(pool_helper.config, canister_name, canister_id)?;

        if let Some(builder) = pool_helper.builder_pool.get(&info) {
            pool_helper
                .canisters_map
                .insert(0, Arc::new(Canister::new(info, builder)));
            Ok(())
        } else {
            Err(anyhow!(
                "Cannot find builder for canister '{}'.",
                info.get_name().to_string()
            ))
        }
    }

    pub fn load(
        env: &dyn Environment,
        generate_cid: bool,
        canister_names: &[String],
    ) -> DfxResult<Self> {
        let logger = env.get_logger().new(slog::o!());
        let config = env
            .get_config()
            .ok_or_else(|| anyhow!("Cannot find dfx configuration file in the current working directory. Did you forget to create one?"))?;

        let mut canisters_map = Vec::new();

        let mut pool_helper = PoolConstructHelper {
            config: &config,
            builder_pool: BuilderPool::new(env)?,
            canister_id_store: CanisterIdStore::for_env(env)?,
            generate_cid,
            canisters_map: &mut canisters_map,
        };

        for canister_name in canister_names {
            CanisterPool::insert(canister_name, &mut pool_helper)?;
        }

        Ok(CanisterPool {
            canisters: canisters_map,
            logger,
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
            id_set.insert(canister_id, graph.add_node(canister_id));
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
            let message = match graph.node_weight(err.node_id()) {
                Some(canister_id) => match self.get_canister_info(canister_id) {
                    Some(info) => info.get_name().to_string(),
                    None => format!("<{}>", canister_id.to_text()),
                },
                None => "<Unknown>".to_string(),
            };
            Err(DfxError::new(BuildError::DependencyError(format!(
                "Found circular dependency: {}",
                message
            ))))
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

        build_canister_js(&canister.canister_id(), &canister.info)?;

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
    ) -> DfxResult<Vec<Result<&BuildOutput, BuildError>>> {
        self.step_prebuild_all(&build_config)
            .map_err(|e| DfxError::new(BuildError::PreBuildAllStepFailed(Box::new(e))))?;

        let graph = self.build_dependencies_graph()?;
        let nodes = petgraph::algo::toposort(&graph, None).map_err(|cycle| {
            let message = match graph.node_weight(cycle.node_id()) {
                Some(canister_id) => match self.get_canister_info(canister_id) {
                    Some(info) => info.get_name().to_string(),
                    None => format!("<{}>", canister_id.to_text()),
                },
                None => "<Unknown>".to_string(),
            };
            BuildError::DependencyError(format!("Found circular dependency: {}", message))
        })?;
        let order: Vec<CanisterId> = nodes
            .iter()
            .rev() // Reverse the order, as we have a dependency graph, we want to reverse indices.
            .map(|idx| *graph.node_weight(*idx).unwrap())
            .collect();

        let mut result = Vec::new();
        for canister_id in &order {
            if let Some(canister) = self.get_canister(canister_id) {
                result.push(
                    self.step_prebuild(&build_config, canister)
                        .map_err(|e| BuildError::PreBuildStepFailed(*canister_id, Box::new(e)))
                        .and_then(|_| {
                            self.step_build(&build_config, canister)
                                .map_err(|e| BuildError::BuildStepFailed(*canister_id, Box::new(e)))
                        })
                        .and_then(|o| {
                            self.step_postbuild(&build_config, canister, o)
                                .map_err(|e| {
                                    BuildError::PostBuildStepFailed(*canister_id, Box::new(e))
                                })
                                .map(|_| o)
                        }),
                );
            }
        }

        self.step_postbuild_all(&build_config, &order)
            .map_err(|e| DfxError::new(BuildError::PostBuildAllStepFailed(Box::new(e))))?;

        Ok(result)
    }

    /// Build all canisters, failing with the first that failed the build. Will return
    /// nothing if all succeeded.
    pub fn build_or_fail(&self, build_config: BuildConfig) -> DfxResult<()> {
        let outputs = self.build(build_config)?;

        for output in outputs {
            output.map_err(DfxError::new)?;
        }

        Ok(())
    }
}

fn decode_path_to_str(path: &Path) -> DfxResult<&str> {
    path.to_str().ok_or_else(|| {
        DfxError::new(BuildError::JsBindGenError(format!(
            "Unable to convert output canister js path to a string: {:#?}",
            path
        )))
    })
}

/// Create a canister JavaScript DID and Actor Factory.
fn build_canister_js(canister_id: &CanisterId, canister_info: &CanisterInfo) -> DfxResult {
    let output_did_js_path = canister_info.get_build_idl_path().with_extension("did.js");
    let output_did_ts_path = canister_info
        .get_build_idl_path()
        .with_extension("did.d.ts");

    let (env, ty) = check_candid_file(&canister_info.get_build_idl_path())?;
    let content = ensure_trailing_newline(candid::bindings::javascript::compile(&env, &ty));
    std::fs::write(output_did_js_path, content)?;
    let content = ensure_trailing_newline(candid::bindings::typescript::compile(&env, &ty));
    std::fs::write(output_did_ts_path, content)?;

    let mut language_bindings = assets::language_bindings()?;
    let index_js_path = canister_info.get_index_js_path();
    for f in language_bindings.entries()? {
        let mut file = f?;
        let mut file_contents = String::new();
        file.read_to_string(&mut file_contents)?;

        let new_file_contents = file_contents
            .replace("{canister_id}", &canister_id.to_text())
            .replace("{canister_name}", canister_info.get_name())
            .replace(
                "{canister_name_uppercase}",
                &canister_info.get_name().to_uppercase(),
            );

        match decode_path_to_str(&file.path()?)? {
            "canister.js" => {
                std::fs::write(decode_path_to_str(&index_js_path)?, new_file_contents)?;
            }
            "canisterId.js" => {
                std::fs::write(decode_path_to_str(&index_js_path)?, new_file_contents)?;
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}

fn ensure_trailing_newline(s: String) -> String {
    if s.ends_with('\n') {
        s
    } else {
        let mut s = s;
        s.push('\n');
        s
    }
}
