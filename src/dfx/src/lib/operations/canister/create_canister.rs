use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::provider::get_network_context;
use crate::lib::waiter::waiter_with_timeout;

use anyhow::anyhow;
use ic_types::principal::Principal;
use ic_utils::interfaces::management_canister::attributes::{ComputeAllocation, MemoryAllocation};
use ic_utils::interfaces::ManagementCanister;
use slog::info;
use std::format;
use std::time::Duration;

pub async fn create_canister(
    env: &dyn Environment,
    canister_name: &str,
    timeout: Duration,
    controller: Option<Principal>,
    compute_allocation: Option<ComputeAllocation>,
    memory_allocation: Option<MemoryAllocation>,
) -> DfxResult {
    let log = env.get_logger();
    info!(log, "Creating canister {:?}...", canister_name);

    let _ = env.get_config_or_anyhow();

    let mut canister_id_store = CanisterIdStore::for_env(env)?;

    let network_name = get_network_context()?;

    let non_default_network = if network_name == "local" {
        format!("")
    } else {
        format!("on network {:?} ", network_name)
    };

    match canister_id_store.find(&canister_name) {
        Some(canister_id) => {
            info!(
                log,
                "{:?} canister was already created {}and has canister id: {:?}",
                canister_name,
                non_default_network,
                canister_id.to_text()
            );
            Ok(())
        }
        None => {
            let agent = env
                .get_agent()
                .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;
            let mgr = ManagementCanister::create(agent);
            let (cid,) = mgr
                .create_canister()
                .with_optional_controller(controller)
                .with_optional_compute_allocation(compute_allocation)
                .with_optional_memory_allocation(memory_allocation)
                .call_and_wait(waiter_with_timeout(timeout))
                .await?;
            let canister_id = cid.to_text();
            info!(
                log,
                "{:?} canister created {}with canister id: {:?}",
                canister_name,
                non_default_network,
                canister_id
            );
            canister_id_store.add(&canister_name, canister_id)
        }
    }?;

    Ok(())
}
