use crate::lib::builders::{
    BuildConfig, BuildOutput, CanisterBuilder, IdlBuildOutput, WasmBuildOutput,
};
use crate::lib::canister_info::{CanisterInfo, PullInfo};
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::models::canister::CanisterPool;
use anyhow::anyhow;
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
        unreachable!("call get_pull_build_output directly");
    }

    #[context("Failed to get candid path for pull canister '{}'.", info.get_name())]
    fn get_candid_path(
        &self,
        _pool: &CanisterPool,
        info: &CanisterInfo,
        _config: &BuildConfig,
    ) -> DfxResult<PathBuf> {
        unreachable!("pull canister must provide common_output_idl_path")
    }
}

pub fn get_pull_build_output(
    canister_info: &CanisterInfo,
    pull_info: &PullInfo,
) -> DfxResult<BuildOutput> {
    let canister_id = *pull_info.get_canister_id();
    let output_idl_path = canister_info
        .get_common_output_idl_path()
        .ok_or_else(|| anyhow!("no common output idl path"))?;

    Ok(BuildOutput {
        canister_id,
        // It's impossible to know if the downloaded wasm is gzip or not with only the info in `dfx.json`.
        wasm: WasmBuildOutput::None,
        idl: IdlBuildOutput::File(output_idl_path),
    })
}
