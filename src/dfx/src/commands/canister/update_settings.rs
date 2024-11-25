use crate::lib::canister_logs::log_visibility::LogVisibilityOpt;
use crate::lib::diagnosis::DiagnosedError;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::ic_attributes::{
    get_compute_allocation, get_freezing_threshold, get_log_visibility, get_memory_allocation,
    get_reserved_cycles_limit, get_wasm_memory_limit, CanisterSettings,
};
use crate::lib::operations::canister::{get_canister_status, update_settings};
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::clap::parsers::{
    compute_allocation_parser, freezing_threshold_parser, memory_allocation_parser,
    reserved_cycles_limit_parser, wasm_memory_limit_parser,
};
use anyhow::{bail, Context};
use byte_unit::Byte;
use candid::Principal as CanisterId;
use candid::Principal;
use clap::{ArgAction, Parser};
use dfx_core::cli::ask_for_consent;
use dfx_core::error::identity::InstantiateIdentityFromNameError::GetIdentityPrincipalFailed;
use dfx_core::identity::CallSender;
use fn_error_context::context;
use ic_agent::identity::Identity;
use ic_utils::interfaces::management_canister::StatusCallResult;

/// Update one or more of a canister's settings (i.e its controller, compute allocation, or memory allocation.)
#[derive(Parser, Debug)]
pub struct UpdateSettingsOpts {
    /// Specifies the canister name or id to update. You must specify either canister name/id or the --all option.
    canister: Option<String>,

    /// Updates the settings of all canisters configured in the project dfx.json files.
    #[arg(long, required_unless_present("canister"))]
    all: bool,

    /// Specifies the identity name or the principal of the new controller.
    /// Can be specified more than once, indicating the canister will have multiple controllers.
    /// If any controllers are set with this parameter, any other controllers will be removed.
    #[arg(long, action = ArgAction::Append)]
    set_controller: Option<Vec<String>>,

    /// Add a principal to the list of controllers of the canister.
    #[arg(long, action = ArgAction::Append, conflicts_with("set_controller"))]
    add_controller: Option<Vec<String>>,

    /// Removes a principal from the list of controllers of the canister.
    #[arg(long, action = ArgAction::Append, conflicts_with("set_controller"))]
    remove_controller: Option<Vec<String>>,

    /// Specifies the canister's compute allocation. This should be a percent in the range [0..100]
    #[arg(long, short, value_parser = compute_allocation_parser)]
    compute_allocation: Option<u64>,

    /// Specifies how much memory the canister is allowed to use in total.
    /// This should be a value in the range [0..12 GiB]. Can include units, e.g. "4KiB".
    /// A setting of 0 means the canister will have access to memory on a “best-effort” basis:
    /// It will only be charged for the memory it uses, but at any point in time may stop running
    /// if it tries to allocate more memory when there isn’t space available on the subnet.
    #[arg(long, value_parser = memory_allocation_parser)]
    memory_allocation: Option<Byte>,

    /// Sets the freezing_threshold in SECONDS.
    /// A canister is considered frozen whenever the IC estimates that the canister would be depleted of cycles
    /// before freezing_threshold seconds pass, given the canister's current size and the IC's current cost for storage.
    /// A frozen canister rejects any calls made to it.
    #[arg(long, value_parser = freezing_threshold_parser)]
    freezing_threshold: Option<u64>,

    /// Sets the upper limit of the canister's reserved cycles balance.
    ///
    /// Reserved cycles are cycles that the system sets aside for future use by the canister.
    /// If a subnet's storage exceeds 450 GiB, then every time a canister allocates new storage bytes,
    /// the system sets aside some amount of cycles from the main balance of the canister.
    /// These reserved cycles will be used to cover future payments for the newly allocated bytes.
    /// The reserved cycles are not transferable and the amount of reserved cycles depends on how full the subnet is.
    ///
    /// A setting of 0 means that the canister will trap if it tries to allocate new storage while the subnet's memory usage exceeds 450 GiB.
    #[arg(long, value_parser = reserved_cycles_limit_parser)]
    reserved_cycles_limit: Option<u128>,

    /// Sets a soft limit on the Wasm memory usage of the canister.
    ///
    /// Update calls, timers, heartbeats, installs, and post-upgrades fail if the
    /// Wasm memory usage exceeds this limit. The main purpose of this setting is
    /// to protect against the case when the canister reaches the hard 4GiB
    /// limit.
    ///
    /// Must be a number between 0 B and 256 TiB, inclusive. Can include units, e.g. "4KiB".
    #[arg(long, value_parser = wasm_memory_limit_parser)]
    wasm_memory_limit: Option<Byte>,

    #[command(flatten)]
    log_visibility_opt: Option<LogVisibilityOpt>,

    /// Freezing thresholds above ~1.5 years require this flag as confirmation.
    #[arg(long)]
    confirm_very_long_freezing_threshold: bool,

    /// Skips yes/no checks by answering 'yes'. Such checks can result in loss of control,
    /// so this is not recommended outside of CI.
    #[arg(long, short)]
    yes: bool,

    /// Send request on behalf of the specified principal.
    /// This option only works for a local PocketIC instance.
    #[arg(long)]
    impersonate: Option<Principal>,
}

pub async fn exec(
    env: &dyn Environment,
    opts: UpdateSettingsOpts,
    mut call_sender: &CallSender,
) -> DfxResult {
    let call_sender_override = opts.impersonate.map(CallSender::Impersonate);
    if let Some(ref call_sender_override) = call_sender_override {
        call_sender = call_sender_override;
    };

    // sanity checks
    if let Some(threshold_in_seconds) = opts.freezing_threshold {
        if threshold_in_seconds > 50_000_000 /* ~1.5 years */ && !opts.confirm_very_long_freezing_threshold
        {
            return Err(DiagnosedError::new(
                "The freezing threshold is defined in SECONDS before the canister would run out of cycles, not in cycles.".to_string(),
                "If you truly want to set a freezing threshold that is longer than a year, please run the same command, but with the flag --confirm-very-long-freezing-threshold to confirm you want to do this.".to_string(),
            )).context("Misunderstanding is very likely.");
        }
    }

    fetch_root_key_if_needed(env).await?;

    if !opts.yes && user_is_removing_themselves_as_controller(env, call_sender, &opts)? {
        ask_for_consent("You are trying to remove yourself as a controller of this canister. This may leave this canister un-upgradeable.")?
    }

    let controllers: Option<DfxResult<Vec<_>>> = opts.set_controller.as_ref().map(|controllers| {
        let y: DfxResult<Vec<_>> = controllers
            .iter()
            .map(|controller| controller_to_principal(env, controller))
            .collect::<DfxResult<Vec<_>>>();
        y
    });
    let controllers = controllers
        .transpose()
        .context("Failed to determine all new controllers given in --set-controller.")?;

    let canister_id_store = env.get_canister_id_store()?;

    if let Some(canister_name_or_id) = opts.canister.as_deref() {
        let config = env.get_config()?;
        let config_interface = config.as_ref().map(|config| config.get_config());
        let mut controllers = controllers;
        let canister_id = CanisterId::from_text(canister_name_or_id)
            .or_else(|_| canister_id_store.get(canister_name_or_id))?;
        let textual_cid = canister_id.to_text();
        let canister_name = canister_id_store.get_name(&textual_cid).map(|x| &**x);

        let compute_allocation =
            get_compute_allocation(opts.compute_allocation, config_interface, canister_name)?;
        let memory_allocation =
            get_memory_allocation(opts.memory_allocation, config_interface, canister_name)?;
        let freezing_threshold =
            get_freezing_threshold(opts.freezing_threshold, config_interface, canister_name)?;
        let reserved_cycles_limit =
            get_reserved_cycles_limit(opts.reserved_cycles_limit, config_interface, canister_name)?;
        let wasm_memory_limit =
            get_wasm_memory_limit(opts.wasm_memory_limit, config_interface, canister_name)?;
        let mut current_status: Option<StatusCallResult> = None;
        if let Some(log_visibility) = &opts.log_visibility_opt {
            if log_visibility.require_current_settings() {
                current_status = Some(get_canister_status(env, canister_id, call_sender).await?);
            }
        }
        let log_visibility = get_log_visibility(
            env,
            opts.log_visibility_opt.as_ref(),
            current_status.as_ref(),
            config_interface,
            canister_name,
        )?;
        if let Some(added) = &opts.add_controller {
            if current_status.is_none() {
                current_status = Some(get_canister_status(env, canister_id, call_sender).await?);
            }
            let mut existing_controllers = current_status
                .as_ref()
                .unwrap()
                .settings
                .controllers
                .clone();
            for s in added {
                existing_controllers.push(controller_to_principal(env, s)?);
            }
            controllers = Some(existing_controllers);
        }
        if let Some(removed) = &opts.remove_controller {
            let controllers = if opts.add_controller.is_some() {
                controllers.as_mut().unwrap()
            } else {
                if current_status.is_none() {
                    current_status =
                        Some(get_canister_status(env, canister_id, call_sender).await?);
                }
                controllers.get_or_insert(current_status.unwrap().settings.controllers)
            };
            let removed = removed
                .iter()
                .map(|r| controller_to_principal(env, r))
                .collect::<DfxResult<Vec<_>>>()
                .context("Failed to determine all controllers to remove.")?;
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
            reserved_cycles_limit,
            wasm_memory_limit,
            log_visibility,
        };
        update_settings(env, canister_id, settings, call_sender).await?;
        display_controller_update(&opts, canister_name_or_id);
    } else if opts.all {
        // Update all canister settings.
        let config = env.get_config_or_anyhow()?;
        let config_interface = config.get_config();
        if let Some(canisters) = &config_interface.canisters {
            for canister_name in canisters.keys() {
                let mut controllers = controllers.clone();
                let canister_id = canister_id_store.get(canister_name)?;
                let compute_allocation = get_compute_allocation(
                    opts.compute_allocation,
                    Some(config_interface),
                    Some(canister_name),
                )
                .with_context(|| {
                    format!("Failed to get compute allocation for {canister_name}.")
                })?;
                let memory_allocation = get_memory_allocation(
                    opts.memory_allocation,
                    Some(config_interface),
                    Some(canister_name),
                )
                .with_context(|| format!("Failed to get memory allocation for {canister_name}."))?;
                let freezing_threshold = get_freezing_threshold(
                    opts.freezing_threshold,
                    Some(config_interface),
                    Some(canister_name),
                )
                .with_context(|| {
                    format!("Failed to get freezing threshold for {canister_name}.")
                })?;
                let reserved_cycles_limit = get_reserved_cycles_limit(
                    opts.reserved_cycles_limit,
                    Some(config_interface),
                    Some(canister_name),
                )
                .with_context(|| {
                    format!("Failed to get reserved cycles limit for {canister_name}.")
                })?;
                let wasm_memory_limit = get_wasm_memory_limit(
                    opts.wasm_memory_limit,
                    Some(config_interface),
                    Some(canister_name),
                )
                .with_context(|| format!("Failed to get Wasm memory limit for {canister_name}."))?;
                let mut current_status: Option<StatusCallResult> = None;
                if let Some(log_visibility) = &opts.log_visibility_opt {
                    if log_visibility.require_current_settings() {
                        current_status =
                            Some(get_canister_status(env, canister_id, call_sender).await?);
                    }
                }
                let log_visibility = get_log_visibility(
                    env,
                    opts.log_visibility_opt.as_ref(),
                    current_status.as_ref(),
                    Some(config_interface),
                    Some(canister_name),
                )
                .with_context(|| format!("Failed to get log visibility for {canister_name}."))?;
                if let Some(added) = &opts.add_controller {
                    if current_status.is_none() {
                        current_status =
                            Some(get_canister_status(env, canister_id, call_sender).await?);
                    }
                    let mut existing_controllers = current_status
                        .as_ref()
                        .unwrap()
                        .settings
                        .controllers
                        .clone();
                    for s in added {
                        existing_controllers.push(controller_to_principal(env, s)?);
                    }
                    controllers = Some(existing_controllers);
                }
                if let Some(removed) = &opts.remove_controller {
                    let controllers = if opts.add_controller.is_some() {
                        controllers.as_mut().unwrap()
                    } else {
                        if current_status.is_none() {
                            current_status =
                                Some(get_canister_status(env, canister_id, call_sender).await?);
                        }
                        controllers.get_or_insert(current_status.unwrap().settings.controllers)
                    };
                    let removed = removed
                        .iter()
                        .map(|r| controller_to_principal(env, r))
                        .collect::<DfxResult<Vec<_>>>()
                        .context("Failed to determine all controllers to remove.")?;
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
                    reserved_cycles_limit,
                    wasm_memory_limit,
                    log_visibility,
                };
                update_settings(env, canister_id, settings, call_sender).await?;
                display_controller_update(&opts, canister_name);
            }
        }
    } else {
        bail!("Cannot find canister name.")
    }

    Ok(())
}

fn user_is_removing_themselves_as_controller(
    env: &dyn Environment,
    call_sender: &CallSender,
    opts: &UpdateSettingsOpts,
) -> DfxResult<bool> {
    let caller_principal = match call_sender {
        CallSender::SelectedId => env
            .get_selected_identity_principal()
            .context("Selected identity is not instantiated")?
            .to_string(),
        CallSender::Impersonate(principal) => principal.to_string(),
        CallSender::Wallet(principal) => principal.to_string(),
    };
    let removes_themselves =
        matches!(&opts.remove_controller, Some(remove) if remove.contains(&caller_principal));
    let sets_without_themselves =
        matches!(&opts.set_controller, Some(set) if !set.contains(&caller_principal));
    Ok(removes_themselves || sets_without_themselves)
}

#[context("Failed to convert controller '{}' to a principal", controller)]
fn controller_to_principal(env: &dyn Environment, controller: &str) -> DfxResult<CanisterId> {
    match CanisterId::from_text(controller) {
        Ok(principal) => Ok(principal),
        Err(_) => {
            let current_id = env.get_selected_identity().unwrap();
            if current_id == controller {
                Ok(env.get_selected_identity_principal().unwrap())
            } else {
                let identity_name = controller;
                env.new_identity_manager()?
                    .instantiate_identity_from_name(identity_name, env.get_logger())
                    .and_then(|identity| identity.sender().map_err(GetIdentityPrincipalFailed))
                    .map_err(DfxError::new)
            }
        }
    }
}

fn display_controller_update(opts: &UpdateSettingsOpts, canister_name_or_id: &str) {
    if let Some(new_controllers) = opts.set_controller.as_ref() {
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
