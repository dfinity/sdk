use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ic_attributes::{
    get_compute_allocation, get_freezing_threshold, get_memory_allocation, CanisterSettings,
};
use crate::lib::identity::identity_manager::IdentityManager;
use crate::lib::identity::identity_utils::CallSender;
use crate::lib::identity::Identity;
use crate::lib::operations::canister::create_canister;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::clap::validators::cycle_amount_validator;
use crate::util::clap::validators::{
    compute_allocation_validator, freezing_threshold_validator, memory_allocation_validator,
};
use crate::util::expiry_duration;

use anyhow::{anyhow, bail, Context};
use candid::Principal as CanisterId;
use clap::Parser;
use ic_agent::Identity as _;
use slog::info;

/// Creates an empty canister and associates the assigned Canister ID to the canister name.
#[derive(Parser)]
pub struct CanisterCreateOpts {
    /// Specifies the canister name. Either this or the --all flag are required.
    canister_name: Option<String>,

    /// Creates all canisters configured in dfx.json.
    #[clap(long, required_unless_present("canister-name"))]
    all: bool,

    /// Specifies the initial cycle balance to deposit into the newly created canister.
    /// The specified amount needs to take the canister create fee into account.
    /// This amount is deducted from the wallet's cycle balance.
    #[clap(long, validator(cycle_amount_validator))]
    with_cycles: Option<String>,

    /// Specifies the identity name or the principal of the new controller.
    #[clap(long, multiple_occurrences(true))]
    controller: Option<Vec<String>>,

    /// Specifies the canister's compute allocation. This should be a percent in the range [0..100]
    #[clap(long, short('c'), validator(compute_allocation_validator))]
    compute_allocation: Option<String>,

    /// Specifies how much memory the canister is allowed to use in total.
    /// This should be a value in the range [0..12 GiB].
    /// A setting of 0 means the canister will have access to memory on a “best-effort” basis:
    /// It will only be charged for the memory it uses, but at any point in time may stop running
    /// if it tries to allocate more memory when there isn’t space available on the subnet.
    #[clap(long, validator(memory_allocation_validator))]
    memory_allocation: Option<String>,

    #[clap(long, validator(freezing_threshold_validator), hide(true))]
    freezing_threshold: Option<String>,

    /// Performs the call with the user Identity as the Sender of messages.
    /// Bypasses the Wallet canister.
    #[clap(long)]
    no_wallet: bool,
}

pub async fn exec(
    env: &dyn Environment,
    opts: CanisterCreateOpts,
    mut call_sender: &CallSender,
) -> DfxResult {
    let config = env.get_config_or_anyhow()?;
    let timeout = expiry_duration();

    fetch_root_key_if_needed(env).await?;

    let with_cycles = opts.with_cycles.as_deref();

    let config_interface = config.get_config();
    let network = env.get_network_descriptor();

    let proxy_sender;
    if !opts.no_wallet && !matches!(call_sender, CallSender::Wallet(_)) {
        let wallet = Identity::get_or_create_wallet_canister(
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
                                IdentityManager::new(env)?
                                    .instantiate_identity_from_name(identity_name)
                                    .and_then(|identity| {
                                        identity.sender().map_err(|err| anyhow!(err))
                                    })
                            }
                        }
                    },
                )
                .collect::<DfxResult<Vec<_>>>()
        })
        .transpose()
        .context("Failed to determine controllers.")?;

    if let Some(canister_name) = opts.canister_name.as_deref() {
        let canister_is_remote = config
            .get_config()
            .is_remote_canister(canister_name, &network.name)?;
        if canister_is_remote {
            bail!("Canister '{}' is a remote canister on network '{}', and cannot be created from here.", canister_name, &network.name)
        }
        let compute_allocation = get_compute_allocation(
            opts.compute_allocation.clone(),
            Some(config_interface),
            Some(canister_name),
        )
        .with_context(|| format!("Failed to read compute allocation of {}.", canister_name))?;
        let memory_allocation = get_memory_allocation(
            opts.memory_allocation.clone(),
            Some(config_interface),
            Some(canister_name),
        )
        .with_context(|| format!("Failed to read memory allocation of {}.", canister_name))?;
        let freezing_threshold = get_freezing_threshold(
            opts.freezing_threshold.clone(),
            Some(config_interface),
            Some(canister_name),
        )
        .with_context(|| format!("Failed to read freezing threshold of {}.", canister_name))?;
        create_canister(
            env,
            canister_name,
            timeout,
            with_cycles,
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
                    opts.compute_allocation.clone(),
                    Some(config_interface),
                    Some(canister_name),
                )
                .with_context(|| {
                    format!("Failed to read compute allocation of {}.", canister_name)
                })?;
                let memory_allocation = get_memory_allocation(
                    opts.memory_allocation.clone(),
                    Some(config_interface),
                    Some(canister_name),
                )
                .with_context(|| {
                    format!("Failed to read memory allocation of {}.", canister_name)
                })?;
                let freezing_threshold = get_freezing_threshold(
                    opts.freezing_threshold.clone(),
                    Some(config_interface),
                    Some(canister_name),
                )
                .with_context(|| {
                    format!("Failed to read freezing threshold of {}.", canister_name)
                })?;
                create_canister(
                    env,
                    canister_name,
                    timeout,
                    with_cycles,
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
        }
        Ok(())
    } else {
        unreachable!()
    }
}
