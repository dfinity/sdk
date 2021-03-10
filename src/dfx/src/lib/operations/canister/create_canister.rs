use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_utils::{wallet_for_call_sender, CallSender};
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
    with_cycles: Option<&str>,
    call_sender: &CallSender,
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

            let agent = env
                .get_agent()
                .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;
            let mgr = ManagementCanister::create(agent);
            let cid = match call_sender {
                CallSender::SelectedId => {
                    if network.is_ic {
                        // Provisional commands are whitelisted on production
                        mgr.create_canister()
                            .call_and_wait(waiter_with_timeout(timeout))
                            .await?
                            .0
                    } else {
                        // amount has been validated by cycle_amount_validator
                        let cycles = with_cycles.and_then(|amount| amount.parse::<u64>().ok());
                        mgr.provisional_create_canister_with_cycles(cycles)
                            .call_and_wait(waiter_with_timeout(timeout))
                            .await?
                            .0
                    }
                }
                CallSender::Wallet(some_id) | CallSender::SelectedIdWallet(some_id) => {
                    let wallet = wallet_for_call_sender(env, call_sender, some_id, true).await?;
                    if network.is_ic {
                        // Provisional commands are whitelisted on production
                        let (create_result,): (ic_utils::interfaces::wallet::CreateResult,) =
                            wallet
                                .call_forward(mgr.update_("create_canister").build(), 0)?
                                .call_and_wait(waiter_with_timeout(timeout))
                                .await?;
                        create_result.canister_id
                    } else {
                        // amount has been validated by cycle_amount_validator
                        let cycles = with_cycles
                            .map_or(1000000000001_u64, |amount| amount.parse::<u64>().unwrap());
                        wallet
                            .wallet_create_canister(cycles, None)
                            .call_and_wait(waiter_with_timeout(timeout))
                            .await?
                            .0
                            .canister_id
                    }
                }
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
