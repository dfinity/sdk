use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ic_attributes::CanisterSettings;
use crate::lib::identity::identity_utils::CallSender;
use crate::lib::identity::Identity;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::provider::get_network_context;
use crate::lib::waiter::waiter_with_timeout;

use anyhow::{anyhow, bail, Context};
use fn_error_context::context;
use ic_agent::agent_error::HttpErrorPayload;
use ic_agent::AgentError;
use ic_utils::interfaces::ManagementCanister;
use slog::info;
use std::format;
use std::time::Duration;

// The cycle fee for create request is 0.1T cycles.
const CANISTER_CREATE_FEE: u128 = 100_000_000_000_u128;
// We do not know the minimum cycle balance a canister should have.
// For now create the canister with 3T cycle balance.
const CANISTER_INITIAL_CYCLE_BALANCE: u128 = 3_000_000_000_000_u128;

#[context("Failed to create canister '{}'.", canister_name)]
pub async fn create_canister(
    env: &dyn Environment,
    canister_name: &str,
    timeout: Duration,
    with_cycles: Option<&str>,
    call_sender: &CallSender,
    settings: CanisterSettings,
) -> DfxResult {
    let log = env.get_logger();
    info!(log, "Creating canister {}...", canister_name);

    let config = env.get_config_or_anyhow()?;

    let mut canister_id_store = CanisterIdStore::for_env(env)?;

    let network_name = get_network_context()?;

    if let Some(remote_canister_id) = config
        .get_config()
        .get_remote_canister_id(canister_name, &network_name)
        .unwrap_or_default()
    {
        bail!(
            "{} canister is remote on network {} and has canister id: {}",
            canister_name,
            network_name,
            remote_canister_id.to_text()
        );
    }

    let non_default_network = if network_name == "local" {
        String::new()
    } else {
        format!("on network {} ", network_name)
    };

    match canister_id_store.find(canister_name) {
        Some(canister_id) => {
            info!(
                log,
                "{} canister was already created {}and has canister id: {}",
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
                    // amount has been validated by cycle_amount_validator, which is u128
                    let cycles = with_cycles.and_then(|amount| amount.parse::<u128>().ok());
                    let mut builder = mgr
                        .create_canister()
                        .as_provisional_create_with_amount(cycles);
                    if let Some(controllers) = settings.controllers {
                        for controller in controllers {
                            builder = builder.with_controller(controller);
                        }
                    };
                    let res = builder
                        .with_optional_compute_allocation(settings.compute_allocation)
                        .with_optional_memory_allocation(settings.memory_allocation)
                        .with_optional_freezing_threshold(settings.freezing_threshold)
                        .call_and_wait(waiter_with_timeout(timeout))
                        .await;
                    if let Err(AgentError::HttpError(HttpErrorPayload { status: 404, .. })) = &res {
                        bail!("In order to create a canister on this network, you must use a wallet in order to allocate cycles to the new canister. \
                            To do this, remove the --no-wallet argument and try again. It is also possible to create a canister on this network \
                            using `dfx ledger create-canister`, but doing so will not associate the created canister with any of the canisters in your project.")
                    }
                    res.context("Canister creation call failed.")?.0
                }
                CallSender::Wallet(wallet_id) => {
                    let wallet = Identity::build_wallet_canister(*wallet_id, env).await?;
                    // amount has been validated by cycle_amount_validator
                    let cycles = with_cycles.map_or(
                        CANISTER_CREATE_FEE + CANISTER_INITIAL_CYCLE_BALANCE,
                        |amount| amount.parse::<u128>().unwrap(),
                    );
                    match wallet
                        .wallet_create_canister(
                            cycles,
                            settings.controllers,
                            settings.compute_allocation,
                            settings.memory_allocation,
                            settings.freezing_threshold,
                            waiter_with_timeout(timeout),
                        )
                        .await
                    {
                        Ok(result) => Ok(result.canister_id),
                        Err(AgentError::WalletUpgradeRequired(s)) => {
                            bail!(format!(
                                "{}\nTo upgrade, run dfx wallet upgrade.",
                                AgentError::WalletUpgradeRequired(s)
                            ));
                        }
                        Err(other) => Err(other),
                    }?
                }
            };
            let canister_id = cid.to_text();
            info!(
                log,
                "{} canister created {}with canister id: {}",
                canister_name,
                non_default_network,
                canister_id
            );
            canister_id_store.add(canister_name, &canister_id)
        }
    }?;

    Ok(())
}
