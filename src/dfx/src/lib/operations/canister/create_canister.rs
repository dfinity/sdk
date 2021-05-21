use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ic_attributes::CanisterSettings;
use crate::lib::identity::identity_utils::CallSender;
use crate::lib::identity::Identity;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::provider::get_network_context;
use crate::lib::waiter::waiter_with_timeout;

use anyhow::anyhow;
use ic_utils::call::AsyncCall;
use ic_utils::interfaces::ManagementCanister;
use ic_types::Principal;
use slog::info;
use std::format;
use std::time::Duration;

// The cycle fee for create request is 1T cycles.
const CANISTER_CREATE_FEE: u64 = 1_000_000_000_000_u64;
// We do not know the minimum cycle balance a canister should have.
// For now create the canister with 10T cycle balance.
const CANISTER_INITIAL_CYCLE_BALANCE: u64 = 10_000_000_000_000_u64;

pub async fn create_canister(
    env: &dyn Environment,
    canister_name: &str,
    timeout: Duration,
    with_cycles: Option<&str>,
    call_sender: &CallSender,
    settings: CanisterSettings,
    effective_canister_id: Option<Principal>,
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
            let cid = match call_sender {
                CallSender::SelectedId => {
                    // amount has been validated by cycle_amount_validator
                    let cycles = with_cycles.and_then(|amount| amount.parse::<u64>().ok());
                    mgr
                        .create_canister()
                        .as_provisional_create_with_amount(cycles)
                        .with_optional_controller(settings.controller)
                        .with_optional_compute_allocation(settings.compute_allocation)
                        .with_optional_memory_allocation(settings.memory_allocation)
                        .with_optional_freezing_threshold(settings.freezing_threshold)
                        .with_optional_effective_canister_id(effective_canister_id)
                        .call_and_wait(waiter_with_timeout(timeout))
                        .await?
                        .0
                }
                CallSender::Wallet(wallet_id) | CallSender::SelectedIdWallet(wallet_id) => {
                    let wallet = Identity::build_wallet_canister(wallet_id.clone(), env)?;
                    // amount has been validated by cycle_amount_validator
                    let cycles = with_cycles.map_or(
                        CANISTER_CREATE_FEE + CANISTER_INITIAL_CYCLE_BALANCE,
                        |amount| amount.parse::<u64>().unwrap(),
                    );
                    wallet
                        .wallet_create_canister(
                            cycles,
                            settings.controller,
                            settings.compute_allocation,
                            settings.memory_allocation,
                            settings.freezing_threshold,
                        )
                        .call_and_wait(waiter_with_timeout(timeout))
                        .await?
                        .0
                        .map_err(|err| anyhow!(err))?
                        .canister_id
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
