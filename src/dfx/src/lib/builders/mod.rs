use crate::config::dfinity::{ConfigInterface, Profile};
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::models::canister::CanisterPool;
use ic_agent::CanisterId;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

mod motoko;

#[derive(Debug)]
pub enum WasmBuildOutput {
    // Wasm(Vec<u8>),
    File(PathBuf),
}

#[derive(Debug)]
pub enum IdlBuildOutput {
    // IDLProg(IDLProg),
    File(PathBuf),
}

/// The output of a build.
pub struct BuildOutput {
    pub canister_id: CanisterId,
    pub wasm: WasmBuildOutput,
    pub idl: IdlBuildOutput,
}

/// A stateless canister builder. This is meant to not keep any state and be passed everything.
pub trait CanisterBuilder {
    /// Returns the dependencies of this canister, if any. This should not be a transitive
    /// list.
    fn get_dependencies(
        &self,
        _pool: &CanisterPool,
        _info: &CanisterInfo,
    ) -> DfxResult<Vec<CanisterId>> {
        Ok(Vec::new())
    }

    /// Whether or not the canister info can be built using this builder.
    fn can_build(&self, info: &CanisterInfo) -> bool;

    /// Build a canister. The canister contains all information related to a single canister,
    /// while the config contains information related to this particular build.
    fn build(
        &self,
        pool: &CanisterPool,
        info: &CanisterInfo,
        config: &BuildConfig,
    ) -> DfxResult<BuildOutput>;
}

#[derive(Clone)]
pub struct BuildConfig {
    profile: Profile,
    assets: bool,
    pub generate_id: bool,
    metadata: BTreeMap<String, serde_json::Value>,
}

impl BuildConfig {
    pub fn from_config(config: &ConfigInterface) -> Self {
        BuildConfig {
            profile: config.profile.unwrap_or(Profile::Debug),
            assets: false,
            generate_id: false,
            metadata: BTreeMap::new(),
        }
    }

    pub fn with_assets(self, assets: bool) -> Self {
        Self { assets, ..self }
    }

    pub fn with_generate_id(self, generate_id: bool) -> Self {
        Self {
            generate_id,
            ..self
        }
    }
}

pub struct BuilderPool {
    builders: Vec<Arc<dyn CanisterBuilder>>,
}

impl BuilderPool {
    pub fn new(env: &dyn Environment) -> DfxResult<Self> {
        let mut builders: Vec<Arc<dyn CanisterBuilder>> = Vec::new();
        builders.push(Arc::new(motoko::MotokoBuilder::new(env)?));

        Ok(Self { builders })
    }

    pub fn get(&self, info: &CanisterInfo) -> Option<Arc<dyn CanisterBuilder>> {
        self.builders
            .iter()
            .find(|builder| builder.can_build(info))
            .map(|x| Arc::clone(x))
    }
}
