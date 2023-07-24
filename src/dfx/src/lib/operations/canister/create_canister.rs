use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ic_attributes::CanisterSettings;
use anyhow::{anyhow, bail, Context};
use candid::Principal;
use dfx_core::canister::build_wallet_canister;
use dfx_core::identity::CallSender;
use dfx_core::network::provider::get_network_context;
use fn_error_context::context;
use ic_agent::agent::{RejectCode, RejectResponse};
use ic_agent::agent_error::HttpErrorPayload;
use ic_agent::{Agent, AgentError};
use ic_utils::interfaces::ManagementCanister;
use slog::info;
use std::format;

// The cycle fee for create request is 0.1T cycles.
const CANISTER_CREATE_FEE: u128 = 100_000_000_000_u128;
// We do not know the minimum cycle balance a canister should have.
// For now create the canister with 3T cycle balance.
const CANISTER_INITIAL_CYCLE_BALANCE: u128 = 3_000_000_000_000_u128;

#[context("Failed to create canister '{}'.", canister_name)]
pub async fn create_canister(
    env: &dyn Environment,
    canister_name: &str,
    with_cycles: Option<u128>,
    specified_id: Option<Principal>,
    call_sender: &CallSender,
    settings: CanisterSettings,
) -> DfxResult {
    let log = env.get_logger();
    info!(log, "Creating canister {}...", canister_name);

    let config = env.get_config_or_anyhow()?;

    let mut canister_id_store = env.get_canister_id_store()?;

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

    if let Some(canister_id) = canister_id_store.find(canister_name) {
        info!(
            log,
            "{} canister was already created {}and has canister id: {}",
            canister_name,
            non_default_network,
            canister_id.to_text()
        );
        return Ok(());
    }

    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;
    let cid = match call_sender {
        CallSender::SelectedId => {
            create_with_management_canister(env, agent, with_cycles, specified_id, settings).await
        }
        CallSender::Wallet(wallet_id) => {
            create_with_wallet(agent, wallet_id, with_cycles, settings).await
        }
    }?;
    let canister_id = cid.to_text();
    info!(
        log,
        "{} canister created {}with canister id: {}",
        canister_name,
        non_default_network,
        canister_id
    );
    canister_id_store.add(canister_name, &canister_id)?;

    Ok(())
}

async fn create_with_management_canister(
    env: &dyn Environment,
    agent: &Agent,
    with_cycles: Option<u128>,
    specified_id: Option<Principal>,
    settings: CanisterSettings,
) -> DfxResult<Principal> {
    let mgr = ManagementCanister::create(agent);
    let mut builder = mgr
        .create_canister()
        .as_provisional_create_with_amount(with_cycles)
        .with_effective_canister_id(env.get_effective_canister_id());
    if let Some(sid) = specified_id {
        builder = builder.as_provisional_create_with_specified_id(sid);
    }
    if let Some(controllers) = settings.controllers {
        for controller in controllers {
            builder = builder.with_controller(controller);
        }
    };
    let res = builder
        .with_optional_compute_allocation(settings.compute_allocation)
        .with_optional_memory_allocation(settings.memory_allocation)
        .with_optional_freezing_threshold(settings.freezing_threshold)
        .call_and_wait()
        .await;
    const NEEDS_WALLET: &str = "In order to create a canister on this network, you must use a wallet in order to allocate cycles to the new canister. \
                        To do this, remove the --no-wallet argument and try again. It is also possible to create a canister on this network \
                        using `dfx ledger create-canister`, but doing so will not associate the created canister with any of the canisters in your project.";
    match res {
        Ok((o,)) => Ok(o),
        Err(AgentError::HttpError(HttpErrorPayload { status, .. }))
            if (400..500).contains(&status) =>
        {
            Err(anyhow!(NEEDS_WALLET))
        }
        Err(AgentError::ReplicaError(RejectResponse {
            reject_code: RejectCode::CanisterReject,
            reject_message,
            ..
        })) if reject_message.contains("is not allowed to call ic00 method") => {
            Err(anyhow!(NEEDS_WALLET))
        }
        Err(e) => Err(e).context("Canister creation call failed."),
    }
}

async fn create_with_wallet(
    agent: &Agent,
    wallet_id: &Principal,
    with_cycles: Option<u128>,
    settings: CanisterSettings,
) -> DfxResult<Principal> {
    let wallet = build_wallet_canister(*wallet_id, agent).await?;
    let cycles = with_cycles.unwrap_or(CANISTER_CREATE_FEE + CANISTER_INITIAL_CYCLE_BALANCE);
    match wallet
        .wallet_create_canister(
            cycles,
            settings.controllers,
            settings.compute_allocation,
            settings.memory_allocation,
            settings.freezing_threshold,
        )
        .await
    {
        Ok(result) => Ok(result.canister_id),
        Err(AgentError::WalletUpgradeRequired(s)) => Err(anyhow!(
            "{}\nTo upgrade, run dfx wallet upgrade.",
            AgentError::WalletUpgradeRequired(s)
        )),
        Err(other) => Err(anyhow!(other)),
    }
}
