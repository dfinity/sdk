use crate::lib::builders::{BuildConfig, BuildOutput, BuilderPool, CanisterBuilder};
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{BuildErrorKind, DfxError, DfxResult};
use ic_http_agent::CanisterId;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::BTreeMap;
use std::sync::Arc;

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
}

impl CanisterPool {
    pub fn load(env: &dyn Environment) -> DfxResult<Self> {
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
        })
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

    /// Build all canisters, returning a vector of results of each builds.
    pub fn build(&self, build_config: BuildConfig) -> DfxResult<Vec<DfxResult<BuildOutput>>> {
        // Write all canister IDs if needed.
        for canister in &self.canisters {
            let canister_info = &canister.info;

            if canister_info.get_canister_id().is_none() {
                std::fs::create_dir_all(
                    canister_info
                        .get_canister_id_path()
                        .parent()
                        .expect("Cannot use root."),
                )?;
                std::fs::write(
                    canister_info.get_canister_id_path(),
                    canister_info.generate_canister_id()?.into_blob().0,
                )
                .map_err(DfxError::from)?;
            }
        }

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
                result.push(canister.build(self, &build_config));
            }
        }

        Ok(result)
    }

    /// Build all canisters, failing if any of them failed the build.
    pub fn build_or_fail(&self, build_config: BuildConfig) -> DfxResult<()> {
        let outputs = self.build(build_config)?;

        for output in outputs {
            output?;
        }

        Ok(())
    }
}
