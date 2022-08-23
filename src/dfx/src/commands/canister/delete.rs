use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ic_attributes::CanisterSettings;
use crate::lib::identity::identity_utils::CallSender;
use crate::lib::identity::Identity;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::operations::canister;
use crate::lib::operations::canister::{
    deposit_cycles, start_canister, stop_canister, update_settings,
};
use crate::lib::provider::create_agent_environment;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::waiter::waiter_with_timeout;
use crate::util::assets::wallet_wasm;
use crate::util::{blob_from_arguments, expiry_duration};
use fn_error_context::context;
use ic_utils::call::AsyncCall;
use ic_utils::interfaces::management_canister::attributes::{
    ComputeAllocation, FreezingThreshold, MemoryAllocation,
};
use ic_utils::interfaces::management_canister::CanisterStatus;
use ic_utils::Argument;

use anyhow::{anyhow, Context};
use candid::Principal;
use clap::Parser;
use ic_utils::interfaces::management_canister::builders::InstallMode;
use ic_utils::interfaces::ManagementCanister;
use num_traits::cast::ToPrimitive;
use slog::info;
use std::convert::TryFrom;
use std::time::Duration;

#[allow(deprecated)]
const DANK_PRINCIPAL: Principal =
    Principal::from_slice(&[0, 0, 0, 0, 0, 0xe0, 1, 0x11, 0x01, 0x01]); // Principal: aanaa-xaaaa-aaaah-aaeiq-cai
const WITHDRAWAL_COST: u128 = 10_000_000_000; // Emperically estimated.
const MAX_MEMORY_ALLOCATION: u64 = 8589934592;

/// Deletes a currently stopped canister.
#[derive(Parser)]
pub struct CanisterDeleteOpts {
    /// Specifies the name of the canister to delete.
    /// You must specify either a canister name/id or the --all flag.
    canister: Option<String>,

    /// Deletes all of the canisters configured in the dfx.json file.
    #[clap(long, required_unless_present("canister"))]
    all: bool,

    /// Do not withdrawal cycles, just delete the canister.
    #[clap(long)]
    no_withdrawal: bool,

    /// Withdraw cycles from canister(s) to the specified canister/wallet before deleting.
    #[clap(long, conflicts_with("no-withdrawal"))]
    withdraw_cycles_to_canister: Option<String>,

    /// Withdraw cycles to dank with the current principal.
    #[clap(
        long,
        conflicts_with("withdraw-cycles-to-canister"),
        conflicts_with("no-withdrawal")
    )]
    withdraw_cycles_to_dank: bool,

    /// Withdraw cycles to dank with the given principal.
    #[clap(
        long,
        conflicts_with("withdraw-cycles-to-canister"),
        conflicts_with("no-withdrawal")
    )]
    withdraw_cycles_to_dank_principal: Option<String>,
}

#[context("Failed to delete canister '{}'.", canister)]
async fn delete_canister(
    env: &dyn Environment,
    canister: &str,
    timeout: Duration,
    call_sender: &CallSender,
    no_withdrawal: bool,
    withdraw_cycles_to_canister: Option<String>,
    withdraw_cycles_to_dank: bool,
    withdraw_cycles_to_dank_principal: Option<String>,
) -> DfxResult {
    let log = env.get_logger();
    let mut canister_id_store = CanisterIdStore::for_env(env)?;
    let canister_id =
        Principal::from_text(canister).or_else(|_| canister_id_store.get(canister))?;
    let mut call_sender = call_sender;
    let to_dank = withdraw_cycles_to_dank || withdraw_cycles_to_dank_principal.is_some();

    // Get the canister to transfer the cycles to.
    let target_canister_id = if no_withdrawal {
        None
    } else if to_dank {
        Some(DANK_PRINCIPAL)
    } else {
        match withdraw_cycles_to_canister {
            Some(ref target_canister_id) => {
                Some(Principal::from_text(target_canister_id).with_context(|| {
                    format!("Failed to read canister id {:?}.", target_canister_id)
                })?)
            }
            None => match call_sender {
                CallSender::Wallet(wallet_id) => Some(*wallet_id),
                CallSender::SelectedId => {
                    let network = env.get_network_descriptor();
                    let agent_env = create_agent_environment(env, Some(network.name.clone()))?;
                    let identity_name = agent_env
                        .get_selected_identity()
                        .expect("No selected identity.")
                        .to_string();
                    // If there is no wallet, then do not attempt to withdraw the cycles.
                    Identity::wallet_canister_id(network, &identity_name)?
                }
            },
        }
    };
    let principal = env
        .get_selected_identity_principal()
        .expect("Selected identity not instantiated.");
    let dank_target_principal = match withdraw_cycles_to_dank_principal {
        None => principal,
        Some(principal) => Principal::from_text(&principal)
            .with_context(|| format!("Failed to read principal {:?}.", &principal))?,
    };
    fetch_root_key_if_needed(env).await?;

    if let Some(target_canister_id) = target_canister_id {
        info!(
            log,
            "Beginning withdrawal of cycles to canister {}; on failure try --no-wallet --no-withdrawal.",
            target_canister_id
        );

        // Determine how many cycles we can withdraw.
        let status = canister::get_canister_status(env, canister_id, timeout, call_sender).await?;
        if status.status == CanisterStatus::Stopped {
            let agent = env
                .get_agent()
                .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;
            let mgr = ManagementCanister::create(agent);
            let canister_id =
                Principal::from_text(canister).or_else(|_| canister_id_store.get(canister))?;

            // Set this principal to be a controller and default the other settings.
            let settings = CanisterSettings {
                controllers: Some(vec![principal]),
                compute_allocation: Some(ComputeAllocation::try_from(0u8).unwrap()),
                memory_allocation: Some(MemoryAllocation::try_from(MAX_MEMORY_ALLOCATION).unwrap()),
                freezing_threshold: Some(FreezingThreshold::try_from(0u8).unwrap()),
            };
            info!(log, "Setting the controller to identity princpal.");
            update_settings(env, canister_id, settings, timeout, call_sender).await?;

            // Install a temporary wallet wasm which will transfer the cycles out of the canister before it is deleted.
            let wasm_module = wallet_wasm(env.get_logger())?;
            info!(
                log,
                "Installing temporary wallet in canister {} to enable transfer of cycles.",
                canister
            );
            let args = blob_from_arguments(None, None, None, &None)?;
            let mode = InstallMode::Reinstall;
            let install_builder = mgr
                .install_code(&canister_id, &wasm_module)
                .with_raw_arg(args.to_vec())
                .with_mode(mode);
            let install_result = install_builder
                .build()
                .context("Failed to build InstallCode call.")?
                .call_and_wait(waiter_with_timeout(timeout))
                .await;
            if install_result.is_ok() {
                start_canister(env, canister_id, timeout, &CallSender::SelectedId).await?;
                let status = canister::get_canister_status(
                    env,
                    canister_id,
                    timeout,
                    &CallSender::SelectedId,
                )
                .await?;
                let mut cycles = status.cycles.0.to_u128().unwrap();
                if cycles > WITHDRAWAL_COST {
                    cycles -= WITHDRAWAL_COST;

                    if !to_dank {
                        info!(
                            log,
                            "Transfering {} cycles to canister {}.", cycles, target_canister_id
                        );
                        // Transfer cycles from the source canister to the target canister using the temporary wallet.
                        deposit_cycles(
                            env,
                            target_canister_id,
                            timeout,
                            &CallSender::Wallet(canister_id),
                            cycles,
                        )
                        .await?;
                    } else {
                        info!(
                            log,
                            "Transfering {} cycles to dank principal {}.",
                            cycles,
                            dank_target_principal
                        );
                        let wallet = Identity::build_wallet_canister(canister_id, env).await?;
                        let opt_principal = Some(dank_target_principal);
                        wallet
                            .call(
                                target_canister_id,
                                "mint",
                                Argument::from_candid((opt_principal,)),
                                cycles,
                            )
                            .call_and_wait(waiter_with_timeout(timeout))
                            .await
                            .context("Failed mint call.")?;
                    }
                } else {
                    info!(log, "Too few cycles to withdraw: {}.", cycles);
                }
                stop_canister(env, canister_id, timeout, &CallSender::SelectedId).await?;
            } else {
                info!(
                    log,
                    "Failed to install temporary wallet, deleting without withdrawal."
                );
            }
            call_sender = &CallSender::SelectedId;
        } else {
            info!(
                log,
                "Canister {} must be stopped before it is deleted.", canister_id
            );
        }
    }

    info!(
        log,
        "Deleting code for canister {}, with canister_id {}",
        canister,
        canister_id.to_text(),
    );

    canister::delete_canister(env, canister_id, timeout, call_sender).await?;

    canister_id_store.remove(canister)?;

    Ok(())
}

pub async fn exec(
    env: &dyn Environment,
    opts: CanisterDeleteOpts,
    call_sender: &CallSender,
) -> DfxResult {
    let config = env.get_config_or_anyhow()?;
    let timeout = expiry_duration();

    fetch_root_key_if_needed(env).await?;

    if let Some(canister) = opts.canister.as_deref() {
        delete_canister(
            env,
            canister,
            timeout,
            call_sender,
            opts.no_withdrawal,
            opts.withdraw_cycles_to_canister,
            opts.withdraw_cycles_to_dank,
            opts.withdraw_cycles_to_dank_principal,
        )
        .await
    } else if opts.all {
        if let Some(canisters) = &config.get_config().canisters {
            for canister in canisters.keys() {
                delete_canister(
                    env,
                    canister,
                    timeout,
                    call_sender,
                    opts.no_withdrawal,
                    opts.withdraw_cycles_to_canister.clone(),
                    opts.withdraw_cycles_to_dank,
                    opts.withdraw_cycles_to_dank_principal.clone(),
                )
                .await?;
            }
        }
        Ok(())
    } else {
        unreachable!()
    }
}
