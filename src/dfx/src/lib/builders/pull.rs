use crate::lib::builders::{
    BuildConfig, BuildOutput, CanisterBuilder, IdlBuildOutput, WasmBuildOutput,
};
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::models::canister::CanisterPool;

use candid::Principal as CanisterId;
use fn_error_context::context;

use slog::o;
use std::path::PathBuf;

pub struct PullBuilder {
    _logger: slog::Logger,
}

impl PullBuilder {
    #[context("Failed to create PullBuilder.")]
    pub fn new(env: &dyn Environment) -> DfxResult<Self> {
        Ok(Self {
            _logger: env.get_logger().new(o! {
                "module" => "pull"
            }),
        })
    }
}

impl CanisterBuilder for PullBuilder {
    #[context("Failed to get dependencies for canister '{}'.", info.get_name())]
    fn get_dependencies(
        &self,
        _pool: &CanisterPool,
        info: &CanisterInfo,
    ) -> DfxResult<Vec<CanisterId>> {
        Ok(vec![])
    }

    #[context("Failed to build Pull canister '{}'.", canister_info.get_name())]
    fn build(
        &self,
        _pool: &CanisterPool,
        canister_info: &CanisterInfo,
        _config: &BuildConfig,
    ) -> DfxResult<BuildOutput> {
        Ok(BuildOutput {
            canister_id: candid::Principal::management_canister(),
            wasm: WasmBuildOutput::File(PathBuf::new()),
            idl: IdlBuildOutput::File(PathBuf::new()),
        })
    }

    fn generate_idl(
        &self,
        _pool: &CanisterPool,
        _info: &CanisterInfo,
        _config: &BuildConfig,
    ) -> DfxResult<PathBuf> {
        Ok(PathBuf::new())
    }
}
