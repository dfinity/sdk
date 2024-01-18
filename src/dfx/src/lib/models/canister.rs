use crate::lib::builders::{
    custom_download, BuildConfig, BuildOutput, BuilderPool, CanisterBuilder, IdlBuildOutput,
    WasmBuildOutput,
};
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{BuildError, DfxError, DfxResult};
use crate::lib::metadata::dfx::DfxMetadata;
use crate::lib::metadata::names::{CANDID_ARGS, CANDID_SERVICE, DFX};
use crate::lib::wasm::file::{compress_bytes, read_wasm_module};
use crate::util::{assets, check_candid_file};
use anyhow::{anyhow, bail, Context};
use candid::Principal as CanisterId;
use dfx_core::config::model::canister_id_store::CanisterIdStore;
use dfx_core::config::model::dfinity::{
    CanisterMetadataSection, Config, MetadataVisibility, WasmOptLevel,
};
use fn_error_context::context;
use ic_wasm::metadata::{add_metadata, remove_metadata, Kind};
use ic_wasm::optimize::OptLevel;
use itertools::Itertools;
use petgraph::graph::{DiGraph, NodeIndex};
use rand::{thread_rng, RngCore};
use slog::{error, info, trace, warn, Logger};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashSet};
use std::convert::TryFrom;
use std::ffi::OsStr;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
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
unsafe impl Send for Canister {}
unsafe impl Sync for Canister {}

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

    pub fn get_info(&self) -> &CanisterInfo {
        &self.info
    }

    pub fn canister_id(&self) -> CanisterId {
        self.info.get_canister_id().unwrap()
    }

    // this function is only ever used when build_config.build_mode_check is true
    #[context("Failed to generate random canister id.")]
    pub fn generate_random_canister_id() -> DfxResult<CanisterId> {
        let mut rng = thread_rng();
        let mut v: Vec<u8> = std::iter::repeat(0u8).take(8).collect();
        rng.fill_bytes(v.as_mut_slice());
        CanisterId::try_from(v).context("Failed to convert bytes to canister id.")
    }

    /// Get the build output of a build process. If the output isn't known at this time,
    /// will return [None].
    pub fn get_build_output(&self) -> Option<&BuildOutput> {
        unsafe { (*self.output.as_ptr()).as_ref() }
    }

    #[context("Failed while trying to generate type declarations for '{}'.", self.info.get_name())]
    pub fn generate(&self, pool: &CanisterPool, build_config: &BuildConfig) -> DfxResult {
        self.builder.generate(pool, &self.info, build_config)
    }

    #[context("Failed to post-process wasm of canister '{}'.", self.info.get_name())]
    pub(crate) fn wasm_post_process(
        &self,
        logger: &Logger,
        build_output: &BuildOutput,
    ) -> DfxResult {
        let build_output_wasm_path = match &build_output.wasm {
            WasmBuildOutput::File(p) => p,
            WasmBuildOutput::None => {
                // exclude pull canisters
                return Ok(());
            }
        };
        let wasm_path = self.info.get_build_wasm_path();
        dfx_core::fs::composite::ensure_parent_dir_exists(&wasm_path)?;
        let info = &self.info;
        if info.is_remote() {
            return Ok(());
        }

        let mut m = read_wasm_module(build_output_wasm_path)?;
        let mut modified = false;

        // optimize or shrink
        if let Some(level) = info.get_optimize() {
            trace!(logger, "Optimizing WASM at level {}", level);
            ic_wasm::optimize::optimize(
                &mut m,
                &wasm_opt_level_convert(level),
                false,
                &None,
                false,
            )
            .context("Failed to optimize the WASM module.")?;
            modified = true;
        } else if info.get_shrink() == Some(true)
            || (info.get_shrink().is_none() && (info.is_rust() || info.is_motoko()))
        {
            trace!(logger, "Shrinking WASM");
            ic_wasm::shrink::shrink(&mut m);
            modified = true;
        }

        // metadata
        trace!(logger, "Attaching metadata");
        let mut metadata_sections = info.metadata().sections.clone();
        // Default to write public candid:service unless overwritten
        let mut public_candid = false;
        if (info.is_rust() || info.is_motoko())
            && !metadata_sections.contains_key(CANDID_SERVICE)
            && !metadata_sections.contains_key(CANDID_ARGS)
        {
            public_candid = true;
        }

        if let Some(pullable) = info.get_pullable() {
            let mut dfx_metadata = DfxMetadata::default();
            dfx_metadata.set_pullable(pullable);
            let content = serde_json::to_string_pretty(&dfx_metadata)
                .with_context(|| "Failed to serialize `dfx` metadata.".to_string())?;
            metadata_sections.insert(
                DFX.to_string(),
                CanisterMetadataSection {
                    name: DFX.to_string(),
                    visibility: MetadataVisibility::Public,
                    content: Some(content),
                    ..Default::default()
                },
            );
            public_candid = true;
        }

        if public_candid {
            metadata_sections.insert(
                CANDID_SERVICE.to_string(),
                CanisterMetadataSection {
                    name: CANDID_SERVICE.to_string(),
                    visibility: MetadataVisibility::Public,
                    ..Default::default()
                },
            );

            metadata_sections.insert(
                CANDID_ARGS.to_string(),
                CanisterMetadataSection {
                    name: CANDID_ARGS.to_string(),
                    visibility: MetadataVisibility::Public,
                    ..Default::default()
                },
            );
        }

        for (name, section) in &metadata_sections {
            if section.name == CANDID_SERVICE && info.is_motoko() {
                if let Some(specified_path) = &section.path {
                    check_valid_subtype(&info.get_service_idl_path(), specified_path)?
                } else {
                    // Motoko compiler handles this
                    continue;
                }
            }

            let data = match (section.path.as_ref(), section.content.as_ref()) {
                (None, None) if section.name == CANDID_SERVICE => {
                    dfx_core::fs::read(&info.get_service_idl_path())?
                }
                (None, None) if section.name == CANDID_ARGS => {
                    dfx_core::fs::read(&info.get_init_args_txt_path())?
                }
                (Some(path), None) => dfx_core::fs::read(path)?,
                (None, Some(s)) => s.clone().into_bytes(),
                (Some(_), Some(_)) => {
                    bail!(
                    "Metadata section could not specify path and content at the same time. section: {:?}",
                    &section
                )
                }
                (None, None) => {
                    bail!(
                        "Metadata section must specify a path or content. section: {:?}",
                        &section
                    )
                }
            };

            let visibility = match section.visibility {
                MetadataVisibility::Public => Kind::Public,
                MetadataVisibility::Private => Kind::Private,
            };

            // if the metadata already exists in the wasm with a different visibility,
            // then we have to remove it
            remove_metadata(&mut m, name);

            add_metadata(&mut m, visibility, name, data);
            modified = true;
        }

        // If not modified and not set "gzip" explicitly, copy the wasm file directly so that hash match.
        if !modified && !info.get_gzip() {
            dfx_core::fs::copy(build_output_wasm_path, &wasm_path)?;
            return Ok(());
        }

        let new_bytes = if wasm_path.extension() == Some(OsStr::new("gz")) {
            // gzip
            // Unlike using gzip CLI, the compression below only takes the wasm bytes
            // So as long as the wasm bytes are the same, the gzip file will be the same on different platforms.
            trace!(logger, "Compressing WASM");
            compress_bytes(&m.emit_wasm())?
        } else {
            m.emit_wasm()
        };
        dfx_core::fs::write(&wasm_path, new_bytes)?;

        Ok(())
    }

    pub(crate) fn candid_post_process(
        &self,
        logger: &Logger,
        build_config: &BuildConfig,
        build_output: &BuildOutput,
    ) -> DfxResult {
        trace!(logger, "Post processing candid file");

        let IdlBuildOutput::File(build_idl_path) = &build_output.idl;

        // 1. Copy the complete IDL file to .dfx/local/canisters/NAME/constructor.did.
        let constructor_idl_path = self.info.get_constructor_idl_path();
        dfx_core::fs::composite::ensure_parent_dir_exists(&constructor_idl_path)?;
        dfx_core::fs::copy(build_idl_path, &constructor_idl_path)?;
        dfx_core::fs::set_permissions_readwrite(&constructor_idl_path)?;

        // 2. Separate into service.did and init_args
        let (service_did, init_args) = separate_candid(build_idl_path)?;

        // 3. Save service.did into following places in .dfx/local/:
        //   - canisters/NAME/service.did
        //   - IDL_ROOT/CANISTER_ID.did
        //   - LSP_ROOT/CANISTER_ID.did
        let mut targets = vec![];
        targets.push(self.info.get_service_idl_path());
        let canister_id = self.canister_id();
        targets.push(
            build_config
                .idl_root
                .join(canister_id.to_text())
                .with_extension("did"),
        );
        targets.push(
            build_config
                .lsp_root
                .join(canister_id.to_text())
                .with_extension("did"),
        );

        for target in targets {
            if &target == build_idl_path {
                continue;
            }
            dfx_core::fs::composite::ensure_parent_dir_exists(&target)?;
            dfx_core::fs::write(&target, &service_did)?;
            dfx_core::fs::set_permissions_readwrite(&target)?;
        }

        // 4. Save init_args into .dfx/local/canisters/NAME/init_args.txt
        let init_args_txt_path = self.info.get_init_args_txt_path();
        dfx_core::fs::composite::ensure_parent_dir_exists(&init_args_txt_path)?;
        dfx_core::fs::write(&init_args_txt_path, init_args)?;
        dfx_core::fs::set_permissions_readwrite(&init_args_txt_path)?;
        Ok(())
    }
}

fn wasm_opt_level_convert(opt_level: WasmOptLevel) -> OptLevel {
    use WasmOptLevel::*;
    match opt_level {
        O0 => OptLevel::O0,
        O1 => OptLevel::O1,
        O2 => OptLevel::O2,
        O3 => OptLevel::O3,
        O4 => OptLevel::O4,
        Os => OptLevel::Os,
        Oz => OptLevel::Oz,
        Size => OptLevel::Oz,
        Cycles => OptLevel::O3,
    }
}

fn separate_candid(path: &Path) -> DfxResult<(String, String)> {
    let (env, actor) = check_candid_file(path)?;
    let actor = actor.ok_or_else(|| anyhow!("provided candid file contains no main service"))?;
    if let candid::types::internal::TypeInner::Class(args, ty) = actor.as_ref() {
        use candid_parser::pretty::{
            candid::{compile, pp_ty},
            utils::{concat, enclose},
        };

        let actor = Some(ty.clone());
        let service_did = compile(&env, &actor);
        let doc = concat(args.iter().map(pp_ty), ",");
        let init_args = enclose("(", doc, ")").pretty(80).to_string();
        Ok((service_did, init_args))
    } else {
        // The original candid from builder output doesn't contain init_args
        // Use it directly to avoid items reordering
        let service_did = dfx_core::fs::read_to_string(path)?;
        let init_args = String::from("()");
        Ok((service_did, init_args))
    }
}

#[context("{} is not a valid subtype of {}", specified_idl_path.display(), compiled_idl_path.display())]
fn check_valid_subtype(compiled_idl_path: &Path, specified_idl_path: &Path) -> DfxResult {
    let (mut env, opt_specified) =
        check_candid_file(specified_idl_path).context("Checking specified candid file.")?;
    let specified_type =
        opt_specified.expect("Specified did file should contain some service interface");
    let (env2, opt_compiled) =
        check_candid_file(compiled_idl_path).context("Checking compiled candid file.")?;
    let compiled_type =
        opt_compiled.expect("Compiled did file should contain some service interface");
    let mut gamma = HashSet::new();
    let specified_type = env.merge_type(env2, specified_type);
    candid::types::subtype::subtype(&mut gamma, &env, &compiled_type, &specified_type)?;
    Ok(())
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
    #[context("Failed to insert '{}' into canister pool.", canister_name)]
    fn insert(canister_name: &str, pool_helper: &mut PoolConstructHelper<'_>) -> DfxResult<()> {
        let canister_id = match pool_helper.canister_id_store.find(canister_name) {
            Some(canister_id) => Some(canister_id),
            None if pool_helper.generate_cid => Some(Canister::generate_random_canister_id()?),
            _ => None,
        };
        let info = CanisterInfo::load(pool_helper.config, canister_name, canister_id)?;
        let builder = pool_helper.builder_pool.get(&info);
        pool_helper
            .canisters_map
            .insert(0, Arc::new(Canister::new(info, builder)));
        Ok(())
    }

    #[context("Failed to load canister pool.")]
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
            canister_id_store: env.get_canister_id_store()?,
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

    #[context("Failed to build dependencies graph for canister pool.")]
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

    #[context("Failed step_prebuild_all.")]
    fn step_prebuild_all(&self, log: &Logger, build_config: &BuildConfig) -> DfxResult<()> {
        // moc expects all .did files of dependencies to be in <output_idl_path> with name <canister id>.did.
        // Because some canisters don't get built these .did files have to be copied over manually.
        for canister in self.canisters.iter().filter(|c| {
            build_config
                .canisters_to_build
                .as_ref()
                .map(|cans| !cans.iter().contains(&c.get_name().to_string()))
                .unwrap_or(false)
        }) {
            let maybe_from = if let Some(remote_candid) = canister.info.get_remote_candid() {
                Some(remote_candid)
            } else {
                canister.info.get_output_idl_path()
            };
            if let Some(from) = maybe_from.as_ref() {
                if from.exists() {
                    let to = build_config.idl_root.join(format!(
                        "{}.did",
                        canister.info.get_canister_id()?.to_text()
                    ));
                    trace!(
                        log,
                        "Copying .did for canister {} from {} to {}.",
                        canister.info.get_name(),
                        from.to_string_lossy(),
                        to.to_string_lossy()
                    );
                    dfx_core::fs::composite::ensure_parent_dir_exists(&to)?;
                    dfx_core::fs::copy(from, &to)?;
                    dfx_core::fs::set_permissions_readwrite(&to)?;
                } else {
                    warn!(
                        log,
                        ".did file for canister '{}' does not exist.",
                        canister.get_name(),
                    );
                }
            } else {
                warn!(
                    log,
                    "Canister '{}' has no .did file configured.",
                    canister.get_name()
                );
            }
        }

        // cargo audit
        if self
            .canisters_to_build(build_config)
            .iter()
            .any(|can| can.info.is_rust())
        {
            self.run_cargo_audit()?;
        } else {
            trace!(
                self.logger,
                "No canister of type 'rust' found. Not trying to run 'cargo audit'."
            )
        }

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
        canister.candid_post_process(self.get_logger(), build_config, build_output)?;

        canister.wasm_post_process(self.get_logger(), build_output)?;

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
        for canister in self.canisters_to_build(build_config) {
            let idl_root = &build_config.idl_root;
            let canister_id = canister.canister_id();
            let idl_file_path = idl_root.join(canister_id.to_text()).with_extension("did");

            // Ignore errors (e.g. File Not Found).
            let _ = std::fs::remove_file(idl_file_path);
        }

        Ok(())
    }

    /// Build all canisters, returning a vector of results of each builds.
    #[context("Failed while trying to build all canisters in the canister pool.")]
    pub fn build(
        &self,
        log: &Logger,
        build_config: &BuildConfig,
    ) -> DfxResult<Vec<Result<&BuildOutput, BuildError>>> {
        self.step_prebuild_all(log, build_config)
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

        let canisters_to_build = self.canisters_to_build(build_config);
        let mut result = Vec::new();
        for canister_id in &order {
            if let Some(canister) = self.get_canister(canister_id) {
                if canisters_to_build
                    .iter()
                    .map(|c| c.get_name())
                    .contains(&canister.get_name())
                {
                    trace!(log, "Building canister '{}'.", canister.get_name());
                } else {
                    trace!(log, "Not building canister '{}'.", canister.get_name());
                    continue;
                }
                result.push(
                    self.step_prebuild(build_config, canister)
                        .map_err(|e| {
                            BuildError::PreBuildStepFailed(
                                *canister_id,
                                canister.get_name().to_string(),
                                Box::new(e),
                            )
                        })
                        .and_then(|_| {
                            self.step_build(build_config, canister).map_err(|e| {
                                BuildError::BuildStepFailed(
                                    *canister_id,
                                    canister.get_name().to_string(),
                                    Box::new(e),
                                )
                            })
                        })
                        .and_then(|o| {
                            self.step_postbuild(build_config, canister, o)
                                .map_err(|e| {
                                    BuildError::PostBuildStepFailed(
                                        *canister_id,
                                        canister.get_name().to_string(),
                                        Box::new(e),
                                    )
                                })
                                .map(|_| o)
                        }),
                );
            }
        }

        self.step_postbuild_all(build_config, &order)
            .map_err(|e| DfxError::new(BuildError::PostBuildAllStepFailed(Box::new(e))))?;

        Ok(result)
    }

    /// Build all canisters, failing with the first that failed the build. Will return
    /// nothing if all succeeded.
    #[context("Failed while trying to build all canisters.")]
    pub async fn build_or_fail(&self, log: &Logger, build_config: &BuildConfig) -> DfxResult<()> {
        self.download(build_config).await?;
        let outputs = self.build(log, build_config)?;

        for output in outputs {
            output.map_err(DfxError::new)?;
        }

        Ok(())
    }

    async fn download(&self, build_config: &BuildConfig) -> DfxResult {
        for canister in self.canisters_to_build(build_config) {
            let info = canister.get_info();

            if info.is_custom() {
                custom_download(info, self).await?;
            }
        }
        Ok(())
    }

    /// If `cargo-audit` is installed this runs `cargo audit` and displays any vulnerable dependencies.
    fn run_cargo_audit(&self) -> DfxResult {
        let location = Command::new("cargo")
            .args(["locate-project", "--message-format=plain", "--workspace"])
            .output()
            .context("Failed to run 'cargo locate-project'.")?;
        if !location.status.success() {
            bail!(
                "'cargo locate-project' failed: {}",
                String::from_utf8_lossy(&location.stderr)
            );
        }
        let location = Path::new(std::str::from_utf8(&location.stdout)?);
        if !location
            .parent()
            .expect("Cargo.toml with no parent")
            .join("Cargo.lock")
            .exists()
        {
            warn!(
                self.logger,
                "Skipped audit step as there is no Cargo.lock file."
            );
            return Ok(());
        }
        if Command::new("cargo")
            .arg("audit")
            .arg("--version")
            .output()
            .map(|out| out.status.success())
            .unwrap_or(false)
        {
            info!(
                self.logger,
                "Checking for vulnerabilities in rust canisters."
            );
            let out = Command::new("cargo")
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .arg("audit")
                .output()
                .context("Failed to run 'cargo audit'.")?;
            if out.status.success() {
                info!(self.logger, "Audit found no vulnerabilities.")
            } else {
                error!(self.logger, "Audit found vulnerabilities in rust canisters. Please address these problems as soon as possible!");
            }
        } else {
            warn!(self.logger, "Cannot check for vulnerabilities in rust canisters because cargo-audit is not installed. Please run 'cargo install cargo-audit' so that vulnerabilities can be detected.");
        }
        Ok(())
    }

    pub fn canisters_to_build(&self, build_config: &BuildConfig) -> Vec<&Arc<Canister>> {
        if let Some(canister_names) = &build_config.canisters_to_build {
            self.canisters
                .iter()
                .filter(|can| canister_names.contains(&can.info.get_name().to_string()))
                .collect()
        } else {
            self.canisters.iter().collect()
        }
    }
}

#[context("Failed to decode path to str.")]
fn decode_path_to_str(path: &Path) -> DfxResult<&str> {
    path.to_str().ok_or_else(|| {
        DfxError::new(BuildError::JsBindGenError(format!(
            "Unable to convert output canister js path to a string: {:#?}",
            path
        )))
    })
}

/// Create a canister JavaScript DID and Actor Factory.
#[context("Failed to build canister js for canister '{}'.", canister_info.get_name())]
fn build_canister_js(canister_id: &CanisterId, canister_info: &CanisterInfo) -> DfxResult {
    let output_did_js_path = canister_info
        .get_service_idl_path()
        .with_extension("did.js");
    let output_did_ts_path = canister_info
        .get_service_idl_path()
        .with_extension("did.d.ts");

    let (env, ty) = check_candid_file(&canister_info.get_service_idl_path())?;
    let content = ensure_trailing_newline(candid_parser::bindings::javascript::compile(&env, &ty));
    std::fs::write(&output_did_js_path, content).with_context(|| {
        format!(
            "Failed to write to {}.",
            output_did_js_path.to_string_lossy()
        )
    })?;
    let content = ensure_trailing_newline(candid_parser::bindings::typescript::compile(&env, &ty));
    std::fs::write(&output_did_ts_path, content).with_context(|| {
        format!(
            "Failed to write to {}.",
            output_did_ts_path.to_string_lossy()
        )
    })?;

    let mut language_bindings =
        assets::language_bindings().context("Failed to get language bindings archive.")?;
    let index_js_path = canister_info.get_index_js_path();
    for f in language_bindings
        .entries()
        .context("Failed to read language bindings archive entries.")?
    {
        let mut file = f.context("Failed to read language bindings archive entry.")?;
        let mut file_contents = String::new();
        file.read_to_string(&mut file_contents)
            .context("Failed to read file content.")?;

        let new_file_contents = file_contents
            .replace("{canister_id}", &canister_id.to_text())
            .replace("{canister_name}", canister_info.get_name())
            .replace(
                "{canister_name_uppercase}",
                &canister_info.get_name().to_uppercase(),
            );

        match decode_path_to_str(&file.path()?)? {
            "canister.js" => {
                std::fs::write(decode_path_to_str(&index_js_path)?, new_file_contents)
                    .with_context(|| {
                        format!("Failed to write to {}.", index_js_path.to_string_lossy())
                    })?;
            }
            "canisterId.js" => {
                std::fs::write(decode_path_to_str(&index_js_path)?, new_file_contents)
                    .with_context(|| {
                        format!("Failed to write to {}.", index_js_path.to_string_lossy())
                    })?;
            }
            // skip
            "index.js.hbs" => {}
            "index.d.ts.hbs" => {}
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
