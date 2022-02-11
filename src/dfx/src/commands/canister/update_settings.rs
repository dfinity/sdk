use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ic_attributes::{
    get_compute_allocation, get_freezing_threshold, get_memory_allocation, CanisterSettings,
};
use crate::lib::identity::identity_manager::IdentityManager;
use crate::lib::identity::identity_utils::CallSender;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::operations::canister::{get_canister_status, update_settings};
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::clap::validators::{
    compute_allocation_validator, freezing_threshold_validator, memory_allocation_validator,
};
use crate::util::expiry_duration;

use anyhow::{anyhow, bail};
use clap::{ArgSettings, Parser};
use ic_agent::identity::Identity;
use ic_types::principal::Principal as CanisterId;

/// Update one or more of a canister's settings (i.e its controller, compute allocation, or memory allocation.)
#[derive(Parser)]
pub struct UpdateSettingsOpts {
    /// Specifies the canister name or id to update. You must specify either canister name/id or the --all option.
    canister: Option<String>,

    /// Updates the settings of all canisters configured in the project dfx.json files.
    #[clap(long, required_unless_present("canister"))]
    all: bool,

    /// Specifies the identity name or the principal of the new controller.
    #[clap(long, multiple_occurrences(true))]
    controller: Option<Vec<String>>,

    #[clap(long, multiple_occurrences(true), conflicts_with("controller"))]
    add_controller: Option<Vec<String>>,

    #[clap(long, multiple_occurrences(true), conflicts_with("controller"))]
    remove_controller: Option<Vec<String>>,

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

    #[clap(long, validator(freezing_threshold_validator), setting = ArgSettings::Hidden)]
    freezing_threshold: Option<String>,
}

pub async fn exec(
    env: &dyn Environment,
    opts: UpdateSettingsOpts,
    call_sender: &CallSender,
) -> DfxResult {
    let config = env.get_config_or_anyhow()?;
    let timeout = expiry_duration();
    let config_interface = config.get_config();
    fetch_root_key_if_needed(env).await?;

    let controllers: Option<DfxResult<Vec<_>>> = opts.controller.as_ref().map(|controllers| {
        let y: DfxResult<Vec<_>> = controllers
            .iter()
            .map(|controller| controller_to_principal(env, controller))
            .collect::<DfxResult<Vec<_>>>();
        y
    });
    let controllers = controllers.transpose()?;

    let canister_id_store = CanisterIdStore::for_env(env)?;

    if let Some(canister_name_or_id) = opts.canister.as_deref() {
        let mut controllers = controllers;
        let canister_id = CanisterId::from_text(canister_name_or_id)
            .or_else(|_| canister_id_store.get(canister_name_or_id))?;
        let textual_cid = canister_id.to_text();
        let canister_name = canister_id_store
            .get_name(&textual_cid)
            .ok_or_else(|| anyhow!("Cannot find canister name for id '{}'.", textual_cid))?;

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
        if let Some(added) = &opts.add_controller {
            let status = get_canister_status(env, canister_id, timeout, call_sender).await?;
            let mut existing_controllers = status.settings.controllers;
            for s in added {
                existing_controllers.push(controller_to_principal(env, s)?);
            }
            controllers = Some(existing_controllers);
        }
        if let Some(removed) = &opts.remove_controller {
            let controllers = if opts.add_controller.is_some() {
                controllers.as_mut().unwrap()
            } else {
                let status = get_canister_status(env, canister_id, timeout, call_sender).await?;
                controllers.get_or_insert(status.settings.controllers)
            };
            let removed = removed
                .iter()
                .map(|r| controller_to_principal(env, r))
                .collect::<DfxResult<Vec<_>>>()?;
            for s in removed {
                if let Some(idx) = controllers.iter().position(|x| *x == s) {
                    controllers.swap_remove(idx);
                }
            }
        }
        let settings = CanisterSettings {
            controllers,
            compute_allocation,
            memory_allocation,
            freezing_threshold,
        };
        update_settings(env, canister_id, settings, timeout, call_sender).await?;
        display_controller_update(&opts, canister_name_or_id);
    } else if opts.all {
        // Update all canister settings.
        if let Some(canisters) = &config.get_config().canisters {
            for canister_name in canisters.keys() {
                let mut controllers = controllers.clone();
                let canister_id = canister_id_store.get(canister_name)?;
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
                if let Some(added) = &opts.add_controller {
                    let status =
                        get_canister_status(env, canister_id, timeout, call_sender).await?;
                    let mut existing_controllers = status.settings.controllers;
                    for s in added {
                        existing_controllers.push(controller_to_principal(env, s)?);
                    }
                    controllers = Some(existing_controllers);
                }
                if let Some(removed) = &opts.remove_controller {
                    let controllers = if opts.add_controller.is_some() {
                        controllers.as_mut().unwrap()
                    } else {
                        let status =
                            get_canister_status(env, canister_id, timeout, call_sender).await?;
                        controllers.get_or_insert(status.settings.controllers)
                    };
                    let removed = removed
                        .iter()
                        .map(|r| controller_to_principal(env, r))
                        .collect::<DfxResult<Vec<_>>>()?;
                    for s in removed {
                        if let Some(idx) = controllers.iter().position(|x| *x == s) {
                            controllers.swap_remove(idx);
                        }
                    }
                }
                let settings = CanisterSettings {
                    controllers,
                    compute_allocation,
                    memory_allocation,
                    freezing_threshold,
                };
                update_settings(env, canister_id, settings, timeout, call_sender).await?;
                display_controller_update(&opts, canister_name);
            }
        }
    } else {
        bail!("Cannot find canister name.")
    }

    Ok(())
}

fn controller_to_principal(env: &dyn Environment, controller: &str) -> DfxResult<CanisterId> {
    match CanisterId::from_text(controller) {
        Ok(principal) => Ok(principal),
        Err(_) => {
            let current_id = env.get_selected_identity().unwrap();
            if current_id == controller {
                Ok(env.get_selected_identity_principal().unwrap())
            } else {
                let identity_name = controller;
                IdentityManager::new(env)?
                    .instantiate_identity_from_name(identity_name)
                    .and_then(|identity| identity.sender().map_err(|err| anyhow!(err)))
            }
        }
    }
}

fn display_controller_update(opts: &UpdateSettingsOpts, canister_name_or_id: &str) {
    if let Some(new_controllers) = opts.controller.as_ref() {
        let mut controllers = new_controllers.clone();
        controllers.sort();

        let plural = if controllers.len() > 1 { "s" } else { "" };

        println!(
            "Set controller{} of {:?} to: {}",
            plural,
            canister_name_or_id,
            controllers.join(" ")
        );
    };
    if let Some(added_controllers) = opts.add_controller.as_ref() {
        let mut controllers = added_controllers.clone();
        controllers.sort();

        let plural = if controllers.len() > 1 { "s" } else { "" };

        println!(
            "Added as controller{} of {:?}: {}",
            plural,
            canister_name_or_id,
            controllers.join(" "),
        );
    }
    if let Some(removed_controllers) = opts.remove_controller.as_ref() {
        let mut controllers = removed_controllers.clone();
        controllers.sort();

        let plural = if controllers.len() > 1 { "s" } else { "" };

        println!(
            "Removed from controller{} of {:?}: {}",
            plural,
            canister_name_or_id,
            controllers.join(" "),
        );
    }
}
