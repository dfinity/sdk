use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::provider::get_network_context;
use crate::lib::waiter::waiter_with_timeout;

use anyhow::anyhow;
use ic_utils::call::AsyncCall;
use ic_utils::interfaces::ManagementCanister;
use slog::info;
use std::format;
use std::time::Duration;

pub async fn create_canister(
    env: &dyn Environment,
    canister_name: &str,
    timeout: Duration,
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
            let (cid,) = if network_name == "ic" {
                mgr.create_canister()
                    .call_and_wait(waiter_with_timeout(timeout))
                    .await?
            } else {
                mgr.provisional_create_canister_with_cycles(None)
                    .call_and_wait(waiter_with_timeout(timeout))
                    .await?
            };
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
