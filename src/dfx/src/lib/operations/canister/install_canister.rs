use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::installers::assets::post_install_store_assets;
use crate::lib::waiter::waiter_with_timeout;
use crate::util::expiry_duration;

use ic_agent::{
    Agent, CanisterAttributes, ComputeAllocation, InstallMode, ManagementCanister, MemoryAllocation,
};
use slog::info;

pub async fn install_canister(
    env: &dyn Environment,
    agent: &Agent,
    canister_info: &CanisterInfo,
    compute_allocation: Option<ComputeAllocation>,
    mode: InstallMode,
    memory_allocation: Option<MemoryAllocation>,
    timeout: Option<&str>,
) -> DfxResult {
    let mgr = ManagementCanister::new(agent);
    let log = env.get_logger();
    let canister_id = canister_info.get_canister_id().map_err(|_| {
        DfxError::CannotFindBuildOutputForCanister(canister_info.get_name().to_owned())
    })?;

    info!(
        log,
        "Installing code for canister {}, with canister_id {}",
        canister_info.get_name(),
        canister_id.to_text(),
    );

    let wasm_path = canister_info
        .get_output_wasm_path()
        .expect("Cannot get WASM output path.");
    let wasm = std::fs::read(wasm_path)?;

    let duration = expiry_duration(timeout)?;

    mgr.install_code(
        waiter_with_timeout(duration),
        &canister_id,
        mode,
        &wasm,
        &[],
        &CanisterAttributes {
            compute_allocation,
            memory_allocation,
        },
    )
    .await
    .map_err(DfxError::from)?;

    if canister_info.get_type() == "assets" {
        post_install_store_assets(&canister_info, &agent, duration).await?;
    }

    Ok(())
}