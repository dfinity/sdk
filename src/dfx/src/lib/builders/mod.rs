use crate::config::dfinity::{Config, Profile};
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};

use crate::lib::models::canister::CanisterPool;
use crate::lib::provider::get_network_context;
use ic_agent::CanisterId;
use std::path::PathBuf;
use std::sync::Arc;

mod assets;
mod custom;
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
    /// Returns true if this builder supports building the canister.
    fn supports(&self, info: &CanisterInfo) -> bool;

    /// Returns the dependencies of this canister, if any. This should not be a transitive
    /// list.
    fn get_dependencies(
        &self,
        _pool: &CanisterPool,
        _info: &CanisterInfo,
    ) -> DfxResult<Vec<CanisterId>> {
        Ok(Vec::new())
    }

    fn prebuild(
        &self,
        _pool: &CanisterPool,
        _info: &CanisterInfo,
        _config: &BuildConfig,
    ) -> DfxResult {
        Ok(())
    }

    /// Build a canister. The canister contains all information related to a single canister,
    /// while the config contains information related to this particular build.
    fn build(
        &self,
        pool: &CanisterPool,
        info: &CanisterInfo,
        config: &BuildConfig,
    ) -> DfxResult<BuildOutput>;

    fn postbuild(
        &self,
        _pool: &CanisterPool,
        _info: &CanisterInfo,
        _config: &BuildConfig,
    ) -> DfxResult {
        Ok(())
    }
}

#[derive(Clone)]
pub struct BuildConfig {
    profile: Profile,
    pub skip_frontend: bool,
    pub build_mode_check: bool,

    /// The root of all IDL files.
    pub idl_root: PathBuf,
}

impl BuildConfig {
    pub fn from_config(config: &Config) -> DfxResult<Self> {
        let config_intf = config.get_config();
        let network_name = get_network_context().ok_or_else(|| DfxError::ComputeNetworkNotSet)?;
        let build_root = config.get_temp_path().join(network_name);
        let build_root = build_root.join(config_intf.get_defaults().get_build().get_output());

        Ok(BuildConfig {
            profile: config_intf.profile.unwrap_or(Profile::Debug),
            skip_frontend: false,
            build_mode_check: false,
            idl_root: build_root.join("idl/"),
        })
    }

    pub fn with_skip_frontend(self, skip_frontend: bool) -> Self {
        Self {
            skip_frontend,
            ..self
        }
    }

    pub fn with_build_mode_check(self, build_mode_check: bool) -> Self {
        Self {
            build_mode_check,
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
        builders.push(Arc::new(assets::AssetsBuilder::new(env)?));
        builders.push(Arc::new(custom::CustomBuilder::new(env)?));
        builders.push(Arc::new(motoko::MotokoBuilder::new(env)?));

        Ok(Self { builders })
    }

    pub fn get(&self, info: &CanisterInfo) -> Option<Arc<dyn CanisterBuilder>> {
        self.builders
            .iter()
            .find(|builder| builder.supports(&info))
            .map(|x| Arc::clone(x))
    }
}
