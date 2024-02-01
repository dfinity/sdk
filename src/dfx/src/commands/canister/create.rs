use crate::lib::deps::get_pull_canisters_in_config;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::ic_attributes::{
    get_compute_allocation, get_freezing_threshold, get_memory_allocation,
    get_reserved_cycles_limit, CanisterSettings,
};
use crate::lib::identity::wallet::{get_or_create_wallet_canister, GetOrCreateWalletCanisterError};
use crate::lib::operations::canister::create_canister;
use crate::lib::operations::cycles_ledger::CYCLES_LEDGER_ENABLED;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::clap::parsers::{
    compute_allocation_parser, freezing_threshold_parser, memory_allocation_parser,
    reserved_cycles_limit_parser,
};
use crate::util::clap::parsers::{cycle_amount_parser, icrc_subaccount_parser};
use crate::util::clap::subnet_selection_opt::SubnetSelectionOpt;
use anyhow::{bail, Context};
use byte_unit::Byte;
use candid::Principal as CanisterId;
use clap::{ArgAction, Parser};
use dfx_core::error::identity::instantiate_identity_from_name::InstantiateIdentityFromNameError::GetIdentityPrincipalFailed;
use dfx_core::identity::CallSender;
use ic_agent::Identity as _;
use icrc_ledger_types::icrc1::account::Subaccount;
use slog::{debug, info, warn};

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
    /// This option overwrites the specified_id field in dfx.json.
    #[arg(long, value_name = "PRINCIPAL", conflicts_with = "all")]
    specified_id: Option<CanisterId>,

    /// Specifies the identity name or the principal of the new controller.
    #[arg(long, action = ArgAction::Append)]
    controller: Option<Vec<String>>,

    /// Specifies the canister's compute allocation. This should be a percent in the range [0..100]
    #[arg(long, short('c'), value_parser = compute_allocation_parser)]
    compute_allocation: Option<u64>,

    /// Specifies how much memory the canister is allowed to use in total.
    /// This should be a value in the range [0..12 GiB].
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

    /// Performs the call with the user Identity as the Sender of messages.
    /// Bypasses the Wallet canister.
    #[arg(long)]
    no_wallet: bool,

    /// Transaction timestamp, in nanoseconds, for use in controlling transaction deduplication, default is system time.
    /// https://internetcomputer.org/docs/current/developer-docs/integrations/icrc-1/#transaction-deduplication-
    //TODO(SDK-1331): unhide
    #[arg(long, hide = true, conflicts_with = "all")]
    created_at_time: Option<u64>,

    /// Subaccount of the selected identity to spend cycles from.
    //TODO(SDK-1331): unhide
    #[arg(long, value_parser = icrc_subaccount_parser, hide = true)]
    from_subaccount: Option<Subaccount>,

    #[command(flatten)]
    subnet_selection: SubnetSelectionOpt,
}

pub async fn exec(
    env: &dyn Environment,
    opts: CanisterCreateOpts,
    mut call_sender: &CallSender,
) -> DfxResult {
    let config = env.get_config_or_anyhow()?;

    fetch_root_key_if_needed(env).await?;

    let with_cycles = opts.with_cycles;

    let config_interface = config.get_config();
    let network = env.get_network_descriptor();

    let proxy_sender;
    if opts.specified_id.is_none()
        && !opts.no_wallet
        && !matches!(call_sender, CallSender::Wallet(_))
        && !network.is_playground()
    {
        match get_or_create_wallet_canister(
            env,
            env.get_network_descriptor(),
            env.get_selected_identity().expect("No selected identity"),
        )
        .await
        {
            Ok(wallet) => {
                proxy_sender = CallSender::Wallet(*wallet.canister_id_());
                call_sender = &proxy_sender;
            }
            Err(err) => {
                if CYCLES_LEDGER_ENABLED
                    && matches!(
                        err,
                        GetOrCreateWalletCanisterError::NoWalletConfigured { .. }
                    )
                {
                    debug!(env.get_logger(), "No wallet configured.");
                } else {
                    bail!(err)
                }
            }
        };
    }

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
    let subnet_selection = opts.subnet_selection.into_subnet_selection();

    let pull_canisters_in_config = get_pull_canisters_in_config(env)?;
    if let Some(canister_name) = opts.canister_name.as_deref() {
        if pull_canisters_in_config.contains_key(canister_name) {
            bail!(
                "{0} is a pull dependency. Please deploy it using `dfx deps deploy {0}`",
                canister_name
            );
        }
        let canister_is_remote =
            config_interface.is_remote_canister(canister_name, &network.name)?;
        if canister_is_remote {
            bail!("Canister '{}' is a remote canister on network '{}', and cannot be created from here.", canister_name, &network.name)
        }
        // Specified ID from the command line takes precedence over the one in dfx.json.
        let specified_id = match (
            config_interface.get_specified_id(canister_name)?,
            opts.specified_id,
        ) {
            (Some(specified_id), Some(opts_specified_id)) => {
                if specified_id != opts_specified_id {
                    warn!(
                        env.get_logger(),
                        "Canister '{0}' has a specified ID in dfx.json: {1},
which is different from the one specified in the command line: {2}.
The command line value will be used.",
                        canister_name,
                        specified_id,
                        opts_specified_id
                    );
                }
                Some(opts_specified_id)
            }
            (Some(specified_id), None) => Some(specified_id),
            (None, Some(opts_specified_id)) => Some(opts_specified_id),
            (None, None) => None,
        };
        let compute_allocation = get_compute_allocation(
            opts.compute_allocation,
            Some(config_interface),
            Some(canister_name),
        )
        .with_context(|| format!("Failed to read compute allocation of {}.", canister_name))?;
        let memory_allocation = get_memory_allocation(
            opts.memory_allocation,
            Some(config_interface),
            Some(canister_name),
        )
        .with_context(|| format!("Failed to read memory allocation of {}.", canister_name))?;
        let freezing_threshold = get_freezing_threshold(
            opts.freezing_threshold,
            Some(config_interface),
            Some(canister_name),
        )
        .with_context(|| format!("Failed to read freezing threshold of {}.", canister_name))?;
        let reserved_cycles_limit = get_reserved_cycles_limit(
            opts.reserved_cycles_limit,
            Some(config_interface),
            Some(canister_name),
        )
        .with_context(|| format!("Failed to read reserved cycles limit of {}.", canister_name))?;
        create_canister(
            env,
            canister_name,
            with_cycles,
            specified_id,
            call_sender,
            opts.from_subaccount,
            CanisterSettings {
                controllers,
                compute_allocation,
                memory_allocation,
                freezing_threshold,
                reserved_cycles_limit,
            },
            opts.created_at_time,
            subnet_selection,
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
                        "Skipping canister '{}' because it is remote for network '{}'",
                        canister_name,
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
                    format!("Failed to read compute allocation of {}.", canister_name)
                })?;
                let memory_allocation = get_memory_allocation(
                    opts.memory_allocation,
                    Some(config_interface),
                    Some(canister_name),
                )
                .with_context(|| {
                    format!("Failed to read memory allocation of {}.", canister_name)
                })?;
                let freezing_threshold = get_freezing_threshold(
                    opts.freezing_threshold,
                    Some(config_interface),
                    Some(canister_name),
                )
                .with_context(|| {
                    format!("Failed to read freezing threshold of {}.", canister_name)
                })?;
                let reserved_cycles_limit = get_reserved_cycles_limit(
                    opts.reserved_cycles_limit,
                    Some(config_interface),
                    Some(canister_name),
                )
                .with_context(|| {
                    format!("Failed to read reserved cycles limit of {}.", canister_name)
                })?;
                create_canister(
                    env,
                    canister_name,
                    with_cycles,
                    specified_id,
                    call_sender,
                    opts.from_subaccount,
                    CanisterSettings {
                        controllers: controllers.clone(),
                        compute_allocation,
                        memory_allocation,
                        freezing_threshold,
                        reserved_cycles_limit,
                    },
                    opts.created_at_time,
                    subnet_selection.clone(),
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
