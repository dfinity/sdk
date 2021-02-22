use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::Identity;
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
            let network = env
                .get_network_descriptor()
                .expect("No network descriptor.");

            let identity_name = env
                .get_selected_identity()
                .expect("No selected identity.")
                .to_string();

            info!(log, "Creating the canister using the wallet canister...");
            let wallet =
                Identity::get_or_create_wallet_canister(env, network, &identity_name, true).await?;
            let cid = if network.is_ic {
                // Provisional commands are whitelisted on production
                let mgr = ManagementCanister::create(
                    env.get_agent()
                        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?,
                );
                let (create_result,): (ic_utils::interfaces::wallet::CreateResult,) = wallet
                    .call_forward(mgr.update_("create_canister").build(), 0)?
                    .call_and_wait(waiter_with_timeout(timeout))
                    .await?;
                create_result.canister_id
            } else {
                wallet
                    .wallet_create_canister(1000000000001_u64, None)
                    .call_and_wait(waiter_with_timeout(timeout))
                    .await?
                    .0
                    .canister_id
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
