// use crate::commands::canister::create_waiter;
use crate::lib::builders::{BuildConfig, BuildOutput, BuilderPool, CanisterBuilder};
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{BuildErrorKind, DfxError, DfxResult};
use delay::Delay;
use ic_agent::CanisterId;
use petgraph::graph::{DiGraph, NodeIndex};
use slog::Logger;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Map;
use std::path::PathBuf;
use tokio::runtime::Runtime;

/// Represents a canister from a DFX project. It can be a virtual Canister.
/// Multiple canister instances can have the same info, but would be differentiated
/// by their IDs.
/// Once an instance of a canister is built it is immutable. So for comparing
/// two canisters one can use their ID.
pub struct Canister {
    info: CanisterInfo,
    builder: Arc<dyn CanisterBuilder>,
}

impl Canister {
    /// Create a new canister.
    /// This can only be done by a CanisterPool.
    pub(super) fn new(info: CanisterInfo, builder: Arc<dyn CanisterBuilder>) -> Self {
        Self { info, builder }
    }

    pub fn build(&self, pool: &CanisterPool, build_config: &BuildConfig) -> DfxResult<BuildOutput> {
        self.builder.build(pool, &self.info, build_config)
    }

    pub fn get_name(&self) -> &str {
        self.info.get_name()
    }

    pub fn canister_id(&self) -> CanisterId {
        self.info.get_canister_id().unwrap()
    }
}

/// A canister pool is a list of canisters.
pub struct CanisterPool {
    canisters: Vec<Arc<Canister>>,
    logger: Logger,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CanisterManifest {
    pub canisters: Map<String, serde_json::value::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CanManMetadata {
    pub timestamp: String,
    pub canister_id: String,
    pub wasm_path: PathBuf,
    pub candid_path: PathBuf,
}

impl CanisterManifest {
    pub fn add_entry(&mut self, info: &CanisterInfo, cid: CanisterId) -> DfxResult<()> {
        let metadata = CanManMetadata {
            timestamp: "bbbb".to_owned(),
            canister_id: cid.clone().to_text(),
            wasm_path: info.get_output_wasm_path().unwrap(),
            candid_path: info.get_output_idl_path().unwrap(),
        };
        self.canisters.insert(
            info.get_name().to_string(),
            serde_json::to_value(metadata).unwrap(),
        );
        let manifest_json = serde_json::to_string_pretty(&self)?;
        // write the manifest
        std::fs::write(info.get_manifest_path(), manifest_json).map_err(DfxError::from)?;
        Ok(())
    }
}

impl CanisterPool {
    pub fn load(env: &dyn Environment, _generate_id: bool) -> DfxResult<Self> {
        let logger = env.get_logger().new(slog::o!());
        let config = env
            .get_config()
            .ok_or(DfxError::CommandMustBeRunInAProject)?;
        let canisters = config.get_config().canisters.as_ref().ok_or_else(|| {
            DfxError::Unknown("No canisters in the configuration file.".to_string())
        })?;

        let builder_pool = BuilderPool::new(env)?;
        let mut canisters_map = Vec::new();

        for (key, _value) in canisters.iter() {
            let info = CanisterInfo::load(&config, &key)?;

            if let Some(builder) = builder_pool.get(&info) {
                canisters_map.push(Arc::new(Canister::new(info, builder)));
            } else {
                return Err(DfxError::CouldNotFindBuilderForCanister(
                    info.get_name().to_string(),
                ));
            }
        }

        Ok(CanisterPool {
            canisters: canisters_map,
            logger,
        })
    }

    pub fn create_canisters(&self, env: &dyn Environment) -> DfxResult {
        let agent = env
            .get_agent()
            .ok_or(DfxError::CommandMustBeRunInAProject)?;
        let mut runtime = Runtime::new().expect("Unable to create a runtime");
        // check manifest first before getting new can id here
        for canister in &self.canisters {
            let waiter = Delay::builder()
                .throttle(Duration::from_millis(100))
                .timeout(Duration::from_secs(60))
                .build();

            let info = &canister.info;

            let manifest_path = info.get_manifest_path();
            // check if the canister_manifest.json file exists
            if manifest_path.is_file() {
                {
                    let file = std::fs::File::open(info.get_manifest_path()).unwrap();
                    let mut manifest: CanisterManifest = serde_json::from_reader(file).unwrap();

                    match manifest.canisters.get(info.get_name().clone()) {
                        Some(serde_value) => {
                            let metadata: CanManMetadata =
                                serde_json::from_value(serde_value.to_owned()).unwrap();
                            CanisterId::from_text(metadata.canister_id).ok();
                            ()
                        }
                        None => {
                            let cid = runtime.block_on(agent.create_canister_and_wait(waiter))?;
                            info.set_canister_id(cid.clone())?;
                            manifest.add_entry(info, cid.clone())?;
                            ()
                        }
                    }
                }
            } else {
                let cid = runtime.block_on(agent.create_canister_and_wait(waiter))?;
                info.set_canister_id(cid.clone())?;
                let mut manifest = CanisterManifest {
                    canisters: Map::new(),
                };
                manifest.add_entry(info, cid.clone())?;
            }
            slog::debug!(self.logger, "  {} => {}", canister.get_name(), canister.canister_id().to_text());
        }
        Ok(())
    }

    pub fn get_canister(&self, canister_id: &CanisterId) -> Option<&Canister> {
        for c in &self.canisters {
            let info = &c.info;
            if Some(canister_id) == info.get_canister_id().as_ref() {
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

    fn build_dependencies_graph(&self) -> DfxResult<DiGraph<CanisterId, ()>> {
        let mut graph: DiGraph<CanisterId, ()> = DiGraph::new();
        let mut id_set: BTreeMap<CanisterId, NodeIndex<u32>> = BTreeMap::new();

        // Add all the canisters as nodes.
        for canister in &self.canisters {
            let canister_id = canister.canister_id();
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

    fn step_prebuild_all(
        &self,
        _build_config: &BuildConfig,
        _order: &mut Vec<CanisterId>,
    ) -> DfxResult<()> {
        Ok(())
    }

    fn step_prebuild(&self, _build_config: &BuildConfig, _canister: &Canister) -> DfxResult<()> {
        Ok(())
    }

    fn step_build(
        &self,
        build_config: &BuildConfig,
        canister: &Canister,
    ) -> DfxResult<BuildOutput> {
        canister.build(self, build_config)
    }

    fn step_postbuild(&self, build_config: &BuildConfig, canister: &Canister) -> DfxResult<()> {
        // Copy the IDL output file to the proper directory for downstream to pick it up if
        // needed.
        let idl_root = &build_config.idl_root;
        let canister_id = canister.canister_id();
        let idl_file_path = idl_root
            .join(canister_id.to_text().split_off(3))
            .with_extension("did");

        let output_idl_path = canister
            .info
            .get_output_idl_path()
            .ok_or_else(|| DfxError::Unknown("Could not get the IDL path.".to_string()))?;

        std::fs::create_dir_all(idl_file_path.parent().unwrap())?;
        std::fs::copy(&output_idl_path, &idl_file_path)
            .map(|_| {})
            .map_err(DfxError::from)
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
            let idl_file_path = idl_root
                .join(canister_id.to_text().split_off(3))
                .with_extension("did");

            // Ignore errors (e.g. File Not Found).
            let _ = std::fs::remove_file(idl_file_path);
        }

        Ok(())
    }

    /// Build all canisters, returning a vector of results of each builds.
    pub fn build(&self, build_config: BuildConfig) -> DfxResult<Vec<DfxResult<BuildOutput>>> {
        let graph = self.build_dependencies_graph()?;
        let mut order: Vec<CanisterId> = petgraph::algo::toposort(&graph, None)
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

        self.step_prebuild_all(&build_config, &mut order)?;

        let mut result = Vec::new();
        for canister_id in &order {
            if let Some(canister) = self.get_canister(canister_id) {
                result.push(
                    Ok(())
                        .and_then(|_| self.step_prebuild(&build_config, canister))
                        .and_then(|_| self.step_build(&build_config, canister))
                        .and_then(|output| {
                            self.step_postbuild(&build_config, canister).map(|_| output)
                        }),
                );
            }
        }

        self.step_postbuild_all(&build_config, &order)?;

        Ok(result)
    }

    /// Build all canisters, failing with the first that failed the build. Will return
    /// nothing if all succeeded.
    pub fn build_or_fail(&self, build_config: BuildConfig) -> DfxResult<()> {
        let outputs = self.build(build_config)?;

        for output in outputs {
            output?;
        }

        Ok(())
    }
}
