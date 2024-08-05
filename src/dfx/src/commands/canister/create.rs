use crate::lib::deps::get_pull_canisters_in_config;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::ic_attributes::{
    get_compute_allocation, get_freezing_threshold, get_log_visibility, get_memory_allocation,
    get_reserved_cycles_limit, get_wasm_memory_limit, CanisterSettings,
};
use crate::lib::operations::canister::create_canister;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::clap::parsers::{
    compute_allocation_parser, freezing_threshold_parser, log_visibility_parser,
    memory_allocation_parser, reserved_cycles_limit_parser, wasm_memory_limit_parser,
};
use crate::util::clap::parsers::{cycle_amount_parser, icrc_subaccount_parser};
use crate::util::clap::subnet_selection_opt::SubnetSelectionOpt;
use anyhow::{bail, Context};
use byte_unit::Byte;
use candid::Principal as CanisterId;
use clap::{ArgAction, Parser};
use dfx_core::error::identity::InstantiateIdentityFromNameError::GetIdentityPrincipalFailed;
use dfx_core::identity::CallSender;
use ic_agent::Identity as _;
use ic_utils::interfaces::management_canister::LogVisibility;
use icrc_ledger_types::icrc1::account::Subaccount;
use slog::info;

/// Creates an empty canister and associates the assigned Canister ID to the canister name.
#[derive(Parser)]
pub struct CanisterCreateOpts {
    /// Specifies the canister name. Either this or the --all flag are required.
    canister_name: Option<String>,

    /// Creates all canisters configured in dfx.json.
    #[arg(long, required_unless_present("canister_name"))]
    all: bool,

    /// Specifies the initial cycle balance to deposit into the newly created canister.
    /// The specified amount needs to take the canister create fee into account.
    /// This amount is deducted from the wallet's cycle balance.
    #[arg(long, value_parser = cycle_amount_parser)]
    with_cycles: Option<u128>,

    /// Attempts to create the canister with this Canister ID.
    ///
    /// This option only works with non-mainnet replica.
    /// This option implies the --no-wallet flag.
    /// This option takes precedence over the specified_id field in dfx.json.
    #[arg(long, value_name = "PRINCIPAL", conflicts_with = "all")]
    specified_id: Option<CanisterId>,

    /// Specifies the identity name or the principal of the new controller.
    #[arg(long, action = ArgAction::Append)]
    controller: Option<Vec<String>>,

    /// Specifies the canister's compute allocation. This should be a percent in the range [0..100]
    #[arg(long, short('c'), value_parser = compute_allocation_parser)]
    compute_allocation: Option<u64>,

    /// Specifies how much memory the canister is allowed to use in total.
    /// This should be a value in the range [0..12 GiB]. Can include units, e.g. "4KiB".
    /// A setting of 0 means the canister will have access to memory on a “best-effort” basis:
    /// It will only be charged for the memory it uses, but at any point in time may stop running
    /// if it tries to allocate more memory when there isn’t space available on the subnet.
    #[arg(long, value_parser = memory_allocation_parser)]
    memory_allocation: Option<Byte>,

    #[arg(long, value_parser = freezing_threshold_parser, hide = true)]
    freezing_threshold: Option<u64>,

    /// Specifies the upper limit of the canister's reserved cycles balance.
    ///
    /// Reserved cycles are cycles that the system sets aside for future use by the canister.
    /// If a subnet's storage exceeds 450 GiB, then every time a canister allocates new storage bytes,
    /// the system sets aside some amount of cycles from the main balance of the canister.
    /// These reserved cycles will be used to cover future payments for the newly allocated bytes.
    /// The reserved cycles are not transferable and the amount of reserved cycles depends on how full the subnet is.
    ///
    /// A setting of 0 means that the canister will trap if it tries to allocate new storage while the subnet's memory usage exceeds 450 GiB.
    #[arg(long, value_parser = reserved_cycles_limit_parser, hide = true)]
    reserved_cycles_limit: Option<u128>,

    /// Specifies a soft limit on the Wasm memory usage of the canister.
    ///
    /// Update calls, timers, heartbeats, installs, and post-upgrades fail if the
    /// Wasm memory usage exceeds this limit. The main purpose of this setting is
    /// to protect against the case when the canister reaches the hard 4GiB
    /// limit.
    ///
    /// Must be a number between 0 B and 256 TiB, inclusive. Can include units, e.g. "4KiB".
    #[arg(long, value_parser = wasm_memory_limit_parser, hide = true)]
    wasm_memory_limit: Option<Byte>,

    /// Specifies who is allowed to read the canister's logs.
    /// Can be either "controllers" or "public".
    #[arg(long, value_parser = log_visibility_parser)]
    log_visibility: Option<LogVisibility>,

    /// Performs the call with the user Identity as the Sender of messages.
    /// Bypasses the Wallet canister.
    #[arg(long)]
    no_wallet: bool,

    /// Transaction timestamp, in nanoseconds, for use in controlling transaction deduplication, default is system time.
    /// https://internetcomputer.org/docs/current/developer-docs/integrations/icrc-1/#transaction-deduplication-
    #[arg(long, conflicts_with = "all")]
    created_at_time: Option<u64>,

    /// Subaccount of the selected identity to spend cycles from.
    #[arg(long, value_parser = icrc_subaccount_parser)]
    from_subaccount: Option<Subaccount>,

    #[command(flatten)]
    subnet_selection: SubnetSelectionOpt,
}

pub async fn exec(
    env: &dyn Environment,
    opts: CanisterCreateOpts,
    call_sender: &CallSender,
) -> DfxResult {
    let config = env.get_config_or_anyhow()?;

    fetch_root_key_if_needed(env).await?;

    let with_cycles = opts.with_cycles;

    let config_interface = config.get_config();
    let network = env.get_network_descriptor();

    let controllers: Option<Vec<_>> = opts
        .controller
        .clone()
        .map(|controllers| {
            controllers
                .iter()
                .map(
                    |controller| match CanisterId::from_text(controller.clone()) {
                        Ok(principal) => Ok(principal),
                        Err(_) => {
                            let current_id = env.get_selected_identity().unwrap();
                            if current_id == controller {
                                Ok(env.get_selected_identity_principal().unwrap())
                            } else {
                                let identity_name = controller;
                                env.new_identity_manager()?
                                    .instantiate_identity_from_name(identity_name, env.get_logger())
                                    .and_then(|identity| {
                                        identity.sender().map_err(GetIdentityPrincipalFailed)
                                    })
                                    .map_err(DfxError::new)
                            }
                        }
                    },
                )
                .collect::<Result<Vec<_>, DfxError>>()
        })
        .transpose()
        .context("Failed to determine controllers.")?;
    let mut subnet_selection = opts
        .subnet_selection
        .into_subnet_selection_type(env)
        .await?;

    let pull_canisters_in_config = get_pull_canisters_in_config(env)?;
    if let Some(canister_name) = opts.canister_name.as_deref() {
        if pull_canisters_in_config.contains_key(canister_name) {
            bail!("{canister_name} is a pull dependency. Please deploy it using `dfx deps deploy {canister_name}`");
        }
        let canister_is_remote =
            config_interface.is_remote_canister(canister_name, &network.name)?;
        if canister_is_remote {
            bail!("Canister '{canister_name}' is a remote canister on network '{}', and cannot be created from here.", &network.name)
        }
        let compute_allocation = get_compute_allocation(
            opts.compute_allocation,
            Some(config_interface),
            Some(canister_name),
        )
        .with_context(|| format!("Failed to read compute allocation of {canister_name}."))?;
        let memory_allocation = get_memory_allocation(
            opts.memory_allocation,
            Some(config_interface),
            Some(canister_name),
        )
        .with_context(|| format!("Failed to read memory allocation of {canister_name}."))?;
        let freezing_threshold = get_freezing_threshold(
            opts.freezing_threshold,
            Some(config_interface),
            Some(canister_name),
        )
        .with_context(|| format!("Failed to read freezing threshold of {canister_name}."))?;
        let reserved_cycles_limit = get_reserved_cycles_limit(
            opts.reserved_cycles_limit,
            Some(config_interface),
            Some(canister_name),
        )
        .with_context(|| format!("Failed to read reserved cycles limit of {canister_name}."))?;
        let wasm_memory_limit = get_wasm_memory_limit(
            opts.wasm_memory_limit,
            Some(config_interface),
            Some(canister_name),
        )
        .with_context(|| format!("Failed to read Wasm memory limit of {canister_name}."))?;
        let log_visibility = get_log_visibility(
            opts.log_visibility,
            Some(config_interface),
            Some(canister_name),
        )
        .with_context(|| format!("Failed to read log visibility of {canister_name}."))?;
        create_canister(
            env,
            canister_name,
            with_cycles,
            opts.specified_id,
            call_sender,
            opts.no_wallet,
            opts.from_subaccount,
            CanisterSettings {
                controllers,
                compute_allocation,
                memory_allocation,
                freezing_threshold,
                reserved_cycles_limit,
                wasm_memory_limit,
                log_visibility,
            },
            opts.created_at_time,
            &mut subnet_selection,
        )
        .await?;
        Ok(())
    } else if opts.all {
        // Create all canisters.
        if let Some(canisters) = &config_interface.canisters {
            for canister_name in canisters.keys() {
                if pull_canisters_in_config.contains_key(canister_name) {
                    continue;
                }
                let canister_is_remote =
                    config_interface.is_remote_canister(canister_name, &network.name)?;
                if canister_is_remote {
                    info!(
                        env.get_logger(),
                        "Skipping canister '{canister_name}' because it is remote for network '{}'",
                        &network.name,
                    );

                    continue;
                }
                let specified_id = config_interface.get_specified_id(canister_name)?;
                let compute_allocation = get_compute_allocation(
                    opts.compute_allocation,
                    Some(config_interface),
                    Some(canister_name),
                )
                .with_context(|| {
                    format!("Failed to read compute allocation of {canister_name}.")
                })?;
                let memory_allocation = get_memory_allocation(
                    opts.memory_allocation,
                    Some(config_interface),
                    Some(canister_name),
                )
                .with_context(|| format!("Failed to read memory allocation of {canister_name}."))?;
                let freezing_threshold = get_freezing_threshold(
                    opts.freezing_threshold,
                    Some(config_interface),
                    Some(canister_name),
                )
                .with_context(|| {
                    format!("Failed to read freezing threshold of {canister_name}.")
                })?;
                let reserved_cycles_limit = get_reserved_cycles_limit(
                    opts.reserved_cycles_limit,
                    Some(config_interface),
                    Some(canister_name),
                )
                .with_context(|| {
                    format!("Failed to read reserved cycles limit of {canister_name}.")
                })?;
                let wasm_memory_limit = get_wasm_memory_limit(
                    opts.wasm_memory_limit,
                    Some(config_interface),
                    Some(canister_name),
                )
                .with_context(|| format!("Failed to read Wasm memory limit of {canister_name}."))?;
                let log_visibility = get_log_visibility(
                    opts.log_visibility,
                    Some(config_interface),
                    Some(canister_name),
                )
                .with_context(|| format!("Failed to read log visibility of {canister_name}."))?;
                create_canister(
                    env,
                    canister_name,
                    with_cycles,
                    specified_id,
                    call_sender,
                    opts.no_wallet,
                    opts.from_subaccount,
                    CanisterSettings {
                        controllers: controllers.clone(),
                        compute_allocation,
                        memory_allocation,
                        freezing_threshold,
                        reserved_cycles_limit,
                        wasm_memory_limit,
                        log_visibility,
                    },
                    opts.created_at_time,
                    &mut subnet_selection,
                )
                .await?;
            }
            if !pull_canisters_in_config.is_empty() {
                info!(env.get_logger(), "There are pull dependencies defined in dfx.json. Please deploy them using `dfx deps deploy`.");
            }
        }
        Ok(())
    } else {
        unreachable!()
    }
}
