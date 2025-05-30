use crate::lib::cycles_ledger_types::create_canister::{
    CmcCreateCanisterArgs, CmcCreateCanisterError,
};
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ic_attributes::CanisterSettings as DfxCanisterSettings;
use crate::lib::identity::wallet::{get_or_create_wallet_canister, GetOrCreateWalletCanisterError};
use crate::lib::ledger_types::MAINNET_CYCLE_MINTER_CANISTER_ID;
use crate::lib::operations::canister::motoko_playground::reserve_canister_with_playground;
use crate::lib::operations::cycles_ledger::create_with_cycles_ledger;
use crate::lib::telemetry::{CyclesHost, Telemetry};
use crate::util::clap::subnet_selection_opt::SubnetSelectionType;
use anyhow::{anyhow, bail, Context};
use candid::Principal;
use dfx_core::canister::build_wallet_canister;
use dfx_core::identity::CallSender;
use dfx_core::network::provider::get_network_context;
use fn_error_context::context;
use ic_agent::agent::{RejectCode, RejectResponse};
use ic_agent::agent_error::HttpErrorPayload;
use ic_agent::{Agent, AgentError};
use ic_utils::interfaces::management_canister::builders::CanisterSettings;
use ic_utils::interfaces::ManagementCanister;
use ic_utils::Argument;
use icrc_ledger_types::icrc1::account::Subaccount;
use slog::{debug, info, warn};
use std::format;

// The cycle fee for create request is 0.5T cycles.
pub const CANISTER_CREATE_FEE: u128 = 500_000_000_000_u128;
// We do not know the minimum cycle balance a canister should have.
// For now create the canister with 3T cycle balance.
pub const CANISTER_INITIAL_CYCLE_BALANCE: u128 = 3_000_000_000_000_u128;
pub const CMC_CREATE_CANISTER_METHOD: &str = "create_canister";

#[context("Failed to create canister '{}'.", canister_name)]
pub async fn create_canister(
    env: &dyn Environment,
    canister_name: &str,
    with_cycles: Option<u128>,
    specified_id_from_cli: Option<Principal>,
    call_sender: &CallSender,
    no_wallet: bool,
    from_subaccount: Option<Subaccount>,
    settings: DfxCanisterSettings,
    created_at_time: Option<u64>,
    subnet_selection: &mut SubnetSelectionType,
) -> DfxResult {
    let log = env.get_logger();

    let config = env.get_config_or_anyhow()?;
    let config_interface = config.get_config();

    let canister_id_store = env.get_canister_id_store()?;

    let network_name = get_network_context()?;

    if let Some(remote_canister_id) = config_interface
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

    if env.get_network_descriptor().is_playground() {
        return reserve_canister_with_playground(env, canister_name).await;
    }

    // Specified ID from the command line takes precedence over the one in dfx.json.
    let mut specified_id = match (
        config_interface.get_specified_id(canister_name)?,
        specified_id_from_cli,
    ) {
        (Some(specified_id_from_json), Some(specified_id_from_cli)) => {
            if specified_id_from_json != specified_id_from_cli {
                warn!(
                    env.get_logger(),
                    "Canister '{0}' has a specified ID in dfx.json: {1},
which is different from the one specified in the command line: {2}.
The command line value will be used.",
                    canister_name,
                    specified_id_from_json,
                    specified_id_from_cli
                );
            }
            Some(specified_id_from_cli)
        }
        (Some(specified_id_from_json), None) => Some(specified_id_from_json),
        (None, Some(specified_id_from_cli)) => Some(specified_id_from_cli),
        (None, None) => None,
    };

    // If the network is IC mainnet, the specified ID will be overwritten by None.
    if env.get_network_descriptor().is_ic && specified_id.is_some() {
        warn!(
            env.get_logger(),
            "Specified ID is ignored on the IC mainnet."
        );
        specified_id = None;
    }

    // Replace call_sender with wallet canister unless:
    // 1. specified_id is in effect OR
    // 2. --no-wallet is set explicitly OR
    // 3. call_sender is already wallet
    let call_sender =
        if specified_id.is_some() || no_wallet || matches!(call_sender, CallSender::Wallet(_)) {
            *call_sender
        } else {
            match get_or_create_wallet_canister(
                env,
                env.get_network_descriptor(),
                env.get_selected_identity().expect("No selected identity"),
            )
            .await
            {
                Ok(wallet) => CallSender::Wallet(*wallet.canister_id_()),
                Err(err) => {
                    if matches!(
                        err,
                        GetOrCreateWalletCanisterError::NoWalletConfigured { .. }
                    ) {
                        debug!(env.get_logger(), "No wallet configured.");
                        *call_sender
                    } else {
                        bail!(err)
                    }
                }
            }
        };

    let spinner = env.new_spinner(format!("Creating canister {canister_name}...").into());
    let agent = env.get_agent();
    let cid = match call_sender {
        CallSender::SelectedId => {
            let auto_wallet_disabled = std::env::var("DFX_DISABLE_AUTO_WALLET").is_ok();
            let ic_network = env.get_network_descriptor().is_ic;
            if ic_network || auto_wallet_disabled {
                create_with_cycles_ledger(
                    env,
                    agent,
                    canister_name,
                    with_cycles,
                    from_subaccount,
                    settings,
                    created_at_time,
                    subnet_selection,
                )
                .await
            } else {
                create_with_management_canister(env, agent, with_cycles, specified_id, settings)
                    .await
            }
        }
        CallSender::Impersonate(_) => {
            unreachable!("Impersonating sender when creating canisters is not supported.")
        }
        CallSender::Wallet(wallet_id) => {
            create_with_wallet(agent, &wallet_id, with_cycles, settings, subnet_selection).await
        }
    }?;
    spinner.finish_and_clear();
    let canister_id = cid.to_text();
    info!(
        log,
        "{} canister created {}with canister id: {}",
        canister_name,
        non_default_network,
        canister_id
    );
    canister_id_store.add(log, canister_name, &canister_id, None)?;
    Telemetry::allowlist_all_asset_canisters(env.get_config()?.as_deref(), canister_id_store);
    Ok(())
}

async fn create_with_management_canister(
    env: &dyn Environment,
    agent: &Agent,
    with_cycles: Option<u128>,
    specified_id: Option<Principal>,
    settings: DfxCanisterSettings,
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
        .with_optional_reserved_cycles_limit(settings.reserved_cycles_limit)
        .with_optional_wasm_memory_limit(settings.wasm_memory_limit)
        .with_optional_wasm_memory_threshold(settings.wasm_memory_threshold)
        .with_optional_log_visibility(settings.log_visibility)
        .await;
    const NEEDS_WALLET: &str = "In order to create a canister on this network, you must use a wallet in order to allocate cycles to the new canister. \
                        To do this, remove the --no-wallet argument and try again. It is also possible to create a canister on this network \
                        using `dfx ledger create-canister`, but doing so will not associate the created canister with any of the canisters in your project.";
    match res {
        Ok((o,)) => Ok(o),
        Err(AgentError::HttpError(HttpErrorPayload {
            status, content, ..
        })) if (400..500).contains(&status) => {
            let message = String::from_utf8_lossy(&content);
            if message.contains(
                "does not belong to an existing subnet and it is not a mainnet canister ID.",
            ) {
                Err(anyhow!("{message}"))
            } else {
                Err(anyhow!(NEEDS_WALLET))
            }
        }
        Err(AgentError::UncertifiedReject {
            reject:
                RejectResponse {
                    reject_code: RejectCode::CanisterReject,
                    reject_message,
                    ..
                },
            ..
        }) if reject_message.contains("is not allowed to call ic00 method") => {
            Err(anyhow!(NEEDS_WALLET))
        }
        Err(e) => Err(e).context("Canister creation call failed."),
    }
}

async fn create_with_wallet(
    agent: &Agent,
    wallet_id: &Principal,
    with_cycles: Option<u128>,
    settings: DfxCanisterSettings,
    subnet_selection: &SubnetSelectionType,
) -> DfxResult<Principal> {
    Telemetry::set_cycles_host(CyclesHost::CyclesWallet);
    let wallet = build_wallet_canister(*wallet_id, agent).await?;
    let cycles = with_cycles.unwrap_or(CANISTER_CREATE_FEE + CANISTER_INITIAL_CYCLE_BALANCE);

    if let Some(subnet_selection) = subnet_selection.get_user_choice() {
        // `wallet_create_canister` only calls the management canister, which means that canisters only get created on the subnet the wallet is on.
        // For any explicit targeting we need to use the CMC.

        let settings = if settings.controllers.is_some() {
            settings
        } else {
            let identity = agent
                .get_principal()
                .map_err(|err| anyhow!("Failed to get selected identity principal: {err}"))?;
            DfxCanisterSettings {
                controllers: Some(vec![*wallet_id, identity]),
                ..settings
            }
        };

        let call_result: Result<
            (Result<Principal, CmcCreateCanisterError>,),
            ic_agent::AgentError,
        > = wallet
            .call128(
                MAINNET_CYCLE_MINTER_CANISTER_ID,
                CMC_CREATE_CANISTER_METHOD,
                Argument::from_candid((CmcCreateCanisterArgs {
                    settings: Some(CanisterSettings::from(settings)),
                    subnet_selection: Some(subnet_selection),
                },)),
                cycles,
            )
            .await;
        match call_result {
            Ok((Ok(canister_id),)) => Ok(canister_id),
            Ok((Err(err),)) => Err(anyhow!(err)),
            Err(AgentError::WalletUpgradeRequired(s)) => Err(anyhow!(
                "{}\nTo upgrade, run dfx wallet upgrade.",
                AgentError::WalletUpgradeRequired(s)
            )),
            Err(other) => Err(anyhow!(other)),
        }
    } else {
        if settings.reserved_cycles_limit.is_some() {
            bail!(
                "Cannot create a canister using a wallet if the reserved_cycles_limit is set. Please create with --no-wallet or use dfx canister update-settings instead.")
        }
        if settings.wasm_memory_limit.is_some() {
            bail!("Cannot create a canister using a wallet if the wasm_memory_limit is set. Please create with --no-wallet or use dfx canister update-settings instead.")
        }
        if settings.log_visibility.is_some() {
            bail!("Cannot create a canister using a wallet if log_visibility is set. Please create with --no-wallet or use dfx canister update-settings instead.")
        }
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
}
