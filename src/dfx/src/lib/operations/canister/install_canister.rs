use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::installers::assets::post_install_store_assets;
use crate::lib::waiter::waiter_with_timeout;

use anyhow::Context;
use ic_agent::Agent;
use ic_utils::call::AsyncCall;
use ic_utils::interfaces::management_canister::*;
use ic_utils::interfaces::ManagementCanister;
use slog::info;
use std::time::Duration;

#[allow(clippy::too_many_arguments)]
pub async fn install_canister(
    env: &dyn Environment,
    agent: &Agent,
    canister_info: &CanisterInfo,
    args: &[u8],
    compute_allocation: Option<ComputeAllocation>,
    mode: InstallMode,
    memory_allocation: Option<MemoryAllocation>,
    timeout: Duration,
) -> DfxResult {
    let mgr = ManagementCanister::create(agent);
    let log = env.get_logger();
    let canister_id = canister_info.get_canister_id().context(
        format!("Cannot find build output for canister \"{}\". Did you forget to run the \"build\" command?", canister_info.get_name().to_owned()),
    )?;

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

    let install_builder = mgr
        .install_code(&canister_id, &wasm)
        .with_raw_arg(args.to_vec())
        .with_mode(mode);

    let install_builder = if let Some(ca) = compute_allocation {
        install_builder.with_compute_allocation(ca)
    } else {
        install_builder
    };
    let install_builder = if let Some(ma) = memory_allocation {
        install_builder.with_memory_allocation(ma)
    } else {
        install_builder
    };

    install_builder
        .build()?
        .call_and_wait(waiter_with_timeout(timeout))
        .await?;

    if canister_info.get_type() == "assets" {
        post_install_store_assets(&canister_info, &agent, timeout).await?;
    }

    Ok(())
}
