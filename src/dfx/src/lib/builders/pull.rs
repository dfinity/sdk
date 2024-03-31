use crate::lib::builders::{BuildOutput, IdlBuildOutput, WasmBuildOutput};
use crate::lib::canister_info::{CanisterInfo, PullInfo};
use crate::lib::error::DfxResult;
use anyhow::anyhow;

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
