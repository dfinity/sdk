use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ic_attributes::{
    get_compute_allocation, get_freezing_threshold, get_memory_allocation, CanisterSettings,
};
use crate::lib::identity::identity_manager::IdentityManager;
use crate::lib::identity::identity_utils::CallSender;
use crate::lib::operations::canister::create_canister;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::clap::validators::cycle_amount_validator;
use crate::util::clap::validators::{
    compute_allocation_validator, freezing_threshold_validator, memory_allocation_validator,
};
use crate::util::expiry_duration;

use anyhow::anyhow;
use clap::{ArgSettings, Clap};
use ic_agent::identity::Identity;
use ic_types::principal::Principal as CanisterId;

/// Creates an empty canister on the Internet Computer and
/// associates the Internet Computer assigned Canister ID to the canister name.
#[derive(Clap)]
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
    #[clap(long, multiple(true), number_of_values(1))]
    controller: Option<Vec<String>>,

    /// Specifies the canister's compute allocation. This should be a percent in the range [0..100]
    #[clap(long, short('c'), validator(compute_allocation_validator))]
    compute_allocation: Option<String>,

    /// Specifies how much memory the canister is allowed to use in total.
    /// This should be a value in the range [0..12 GiB]
    /// A setting of 0 means the canister will have access to memory on a “best-effort” basis:
    /// It will only be charged for the memory it uses, but at any point in time may stop running
    /// if it tries to allocate more memory when there isn’t space available on the subnet.
    #[clap(long, validator(memory_allocation_validator))]
    memory_allocation: Option<String>,

    #[clap(long, validator(freezing_threshold_validator), setting = ArgSettings::Hidden)]
    freezing_threshold: Option<String>,
}

pub async fn exec(
    env: &dyn Environment,
    opts: CanisterCreateOpts,
    call_sender: &CallSender,
) -> DfxResult {
    let config = env.get_config_or_anyhow()?;
    let timeout = expiry_duration();

    fetch_root_key_if_needed(env).await?;

    let with_cycles = opts.with_cycles.as_deref();

    let config_interface = config.get_config();

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
        .transpose()?;

    if let Some(canister_name) = opts.canister_name.as_deref() {
        let compute_allocation = get_compute_allocation(
            opts.compute_allocation.clone(),
            config_interface,
            canister_name,
        )?;
        let memory_allocation = get_memory_allocation(
            opts.memory_allocation.clone(),
            config_interface,
            canister_name,
        )?;
        let freezing_threshold = get_freezing_threshold(
            opts.freezing_threshold.clone(),
            config_interface,
            canister_name,
        )?;
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
                let compute_allocation = get_compute_allocation(
                    opts.compute_allocation.clone(),
                    config_interface,
                    canister_name,
                )?;
                let memory_allocation = get_memory_allocation(
                    opts.memory_allocation.clone(),
                    config_interface,
                    canister_name,
                )?;
                let freezing_threshold = get_freezing_threshold(
                    opts.freezing_threshold.clone(),
                    config_interface,
                    canister_name,
                )?;
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
