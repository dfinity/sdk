use crate::lib::deps::get_pull_canisters_in_config;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ic_attributes::{
    get_compute_allocation, get_freezing_threshold, get_memory_allocation, CanisterSettings,
};
use crate::lib::identity::wallet::get_or_create_wallet_canister;
use crate::lib::operations::canister::create_canister;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::clap::parsers::cycle_amount_parser;
use crate::util::clap::parsers::{
    compute_allocation_parser, freezing_threshold_parser, memory_allocation_parser,
};
use byte_unit::Byte;
use dfx_core::error::identity::IdentityError;
use dfx_core::error::identity::IdentityError::GetIdentityPrincipalFailed;
use dfx_core::identity::CallSender;

use anyhow::{bail, Context};
use candid::Principal as CanisterId;
use clap::{ArgAction, Parser};
use ic_agent::Identity as _;
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

    /// Performs the call with the user Identity as the Sender of messages.
    /// Bypasses the Wallet canister.
    #[arg(long)]
    no_wallet: bool,
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
    {
        let wallet = get_or_create_wallet_canister(
            env,
            env.get_network_descriptor(),
            env.get_selected_identity().expect("No selected identity"),
        )
        .await?;
        proxy_sender = CallSender::Wallet(*wallet.canister_id_());
        call_sender = &proxy_sender;
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
                            }
                        }
                    },
                )
                .collect::<Result<Vec<_>, IdentityError>>()
        })
        .transpose()
        .context("Failed to determine controllers.")?;

    let pull_canisters_in_config = get_pull_canisters_in_config(env)?;
    if let Some(canister_name) = opts.canister_name.as_deref() {
        if pull_canisters_in_config.contains_key(canister_name) {
            bail!(
                "{0} is a pull dependency. Please deploy it using `dfx deps deploy {0}`",
                canister_name
            );
        }
        let canister_is_remote = config
            .get_config()
            .is_remote_canister(canister_name, &network.name)?;
        if canister_is_remote {
            bail!("Canister '{}' is a remote canister on network '{}', and cannot be created from here.", canister_name, &network.name)
        }
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
        create_canister(
            env,
            canister_name,
            with_cycles,
            opts.specified_id,
            call_sender,
            CanisterSettings {
                controllers,
                compute_allocation,
                memory_allocation,
                freezing_threshold,
            },
        )
        .await?;
        Ok(())
    } else if opts.all {
        // Create all canisters.
        if let Some(canisters) = &config.get_config().canisters {
            for canister_name in canisters.keys() {
                if pull_canisters_in_config.contains_key(canister_name) {
                    continue;
                }
                let canister_is_remote = config
                    .get_config()
                    .is_remote_canister(canister_name, &network.name)?;
                if canister_is_remote {
                    info!(
                        env.get_logger(),
                        "Skipping canister '{}' because it is remote for network '{}'",
                        canister_name,
                        &network.name,
                    );

                    continue;
                }
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
                create_canister(
                    env,
                    canister_name,
                    with_cycles,
                    None,
                    call_sender,
                    CanisterSettings {
                        controllers: controllers.clone(),
                        compute_allocation,
                        memory_allocation,
                        freezing_threshold,
                    },
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
