use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ic_attributes::CanisterSettings;
use crate::lib::operations::canister;
use crate::lib::operations::canister::{
    deposit_cycles, start_canister, stop_canister, update_settings,
};
use crate::lib::operations::cycles_ledger::wallet_deposit_to_cycles_ledger;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::assets::wallet_wasm;
use crate::util::blob_from_arguments;
use crate::util::clap::parsers::{cycle_amount_parser, icrc_subaccount_parser};
use anyhow::{bail, Context};
use candid::Principal;
use clap::Parser;
use dfx_core::canister::build_wallet_canister;
use dfx_core::cli::ask_for_consent;
use dfx_core::identity::wallet::wallet_canister_id;
use dfx_core::identity::CallSender;
use fn_error_context::context;
use ic_utils::interfaces::management_canister::attributes::FreezingThreshold;
use ic_utils::interfaces::management_canister::builders::InstallMode;
use ic_utils::interfaces::management_canister::CanisterStatus;
use ic_utils::interfaces::ManagementCanister;
use ic_utils::Argument;
use icrc_ledger_types::icrc1::account::{Account, Subaccount};
use num_traits::cast::ToPrimitive;
use slog::{debug, info};
use std::convert::TryFrom;

const DANK_PRINCIPAL: Principal =
    Principal::from_slice(&[0, 0, 0, 0, 0, 0xe0, 1, 0x11, 0x01, 0x01]); // Principal: aanaa-xaaaa-aaaah-aaeiq-cai

// "Couldn't send message" when deleting a canister: increase WITHDRAWAL_COST
const WITHDRAWAL_COST: u128 = 30_000_000_000; // conservative estimate based on mainnet observation

/// Deletes a currently stopped canister.
#[derive(Parser)]
pub struct CanisterDeleteOpts {
    /// Specifies the name of the canister to delete.
    /// You must specify either a canister name/id or the --all flag.
    canister: Option<String>,

    /// Deletes all of the canisters configured in the dfx.json file.
    #[arg(long, required_unless_present("canister"))]
    all: bool,

    /// Do not withdrawal cycles, just delete the canister.
    #[arg(long)]
    no_withdrawal: bool,

    /// Withdraw cycles from canister(s) to the specified canister/wallet before deleting.
    #[arg(long, conflicts_with("no_withdrawal"))]
    withdraw_cycles_to_canister: Option<String>,

    /// Withdraw cycles to dank with the current principal.
    #[arg(
        long,
        conflicts_with("withdraw_cycles_to_canister"),
        conflicts_with("no_withdrawal")
    )]
    withdraw_cycles_to_dank: bool,

    /// Withdraw cycles to dank with the given principal.
    #[arg(
        long,
        conflicts_with("withdraw_cycles_to_canister"),
        conflicts_with("no_withdrawal")
    )]
    withdraw_cycles_to_dank_principal: Option<String>,

    /// Leave this many cycles in the canister when withdrawing cycles.
    #[arg(long, value_parser = cycle_amount_parser, conflicts_with("no_withdrawal"))]
    initial_margin: Option<u128>,

    /// Auto-confirm deletion for a non-stopped canister.
    #[arg(long, short)]
    yes: bool,

    /// Subaccount of the selected identity to deposit cycles to.
    #[arg(long, value_parser = icrc_subaccount_parser)]
    to_subaccount: Option<Subaccount>,
}

#[context("Failed to delete canister '{}'.", canister)]
async fn delete_canister(
    env: &dyn Environment,
    canister: &str,
    call_sender: &CallSender,
    no_withdrawal: bool,
    skip_confirmation: bool,
    withdraw_cycles_to_canister: Option<String>,
    withdraw_cycles_to_dank: bool,
    withdraw_cycles_to_dank_principal: Option<String>,
    to_cycles_ledger_subaccount: Option<Subaccount>,
    initial_margin: Option<u128>,
) -> DfxResult {
    let log = env.get_logger();
    let mut canister_id_store = env.get_canister_id_store()?;
    let (canister_id, canister_name_to_delete) = match Principal::from_text(canister) {
        Ok(canister_id) => (
            canister_id,
            canister_id_store.get_name_in_project(canister).cloned(),
        ),
        Err(_) => (canister_id_store.get(canister)?, Some(canister.to_string())),
    };

    if !env.get_network_descriptor().is_playground() {
        let mut call_sender = call_sender;
        let to_dank = withdraw_cycles_to_dank || withdraw_cycles_to_dank_principal.is_some();

        // Get the canister to transfer the cycles to.
        let withdraw_target = if no_withdrawal {
            WithdrawTarget::NoWithdrawal
        } else if to_dank {
            WithdrawTarget::Dank
        } else {
            match withdraw_cycles_to_canister {
                Some(ref target_canister_id) => {
                    let canister_id =
                        Principal::from_text(target_canister_id).with_context(|| {
                            format!("Failed to read canister id {:?}.", target_canister_id)
                        })?;
                    WithdrawTarget::Canister { canister_id }
                }
                None => match call_sender {
                    CallSender::Wallet(wallet_id) => WithdrawTarget::Canister {
                        canister_id: *wallet_id,
                    },
                    CallSender::Impersonate(_) => {
                        unreachable!(
                            "Impersonating sender when deleting canisters is not supported."
                        )
                    }
                    CallSender::SelectedId => {
                        let network = env.get_network_descriptor();
                        let identity_name = env
                            .get_selected_identity()
                            .expect("No selected identity.")
                            .to_string();
                        // If there is no wallet, then do not attempt to withdraw the cycles.
                        match wallet_canister_id(network, &identity_name)? {
                            Some(canister_id) => WithdrawTarget::Canister { canister_id },
                            None => {
                                let Some(my_principal) = env.get_selected_identity_principal()
                                else {
                                    bail!("Identity has no principal attached")
                                };
                                WithdrawTarget::CyclesLedger {
                                    to: Account {
                                        owner: my_principal,
                                        subaccount: to_cycles_ledger_subaccount,
                                    },
                                }
                            }
                        }
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

        if withdraw_target != WithdrawTarget::NoWithdrawal {
            info!(
                log,
                "Beginning withdrawal of cycles; on failure try --no-wallet --no-withdrawal."
            );

            // Determine how many cycles we can withdraw.
            let status = canister::get_canister_status(env, canister_id, call_sender).await?;
            if status.status != CanisterStatus::Stopped && !skip_confirmation {
                ask_for_consent(&format!(
                    "Canister {canister} has not been stopped. Delete anyway?"
                ))?;
            }
            let agent = env.get_agent();
            let mgr = ManagementCanister::create(agent);
            let canister_id =
                Principal::from_text(canister).or_else(|_| canister_id_store.get(canister))?;

            // Set this principal to be a controller and minimize the freezing threshold to free up as many cycles as possible.
            let settings = CanisterSettings {
                controllers: Some(vec![principal]),
                compute_allocation: None,
                memory_allocation: None,
                freezing_threshold: Some(FreezingThreshold::try_from(0u8).unwrap()),
                reserved_cycles_limit: None,
                wasm_memory_limit: None,
                log_visibility: None,
            };
            info!(log, "Setting the controller to identity principal.");
            update_settings(env, canister_id, settings, call_sender).await?;

            // Install a temporary wallet wasm which will transfer the cycles out of the canister before it is deleted.
            let wasm_module = wallet_wasm(env.get_logger())?;
            info!(
                log,
                "Installing temporary wallet in canister {} to enable transfer of cycles.",
                canister
            );
            let args = blob_from_arguments(None, None, None, None, &None, false, false)?;
            let mode = InstallMode::Reinstall;
            let install_builder = mgr
                .install_code(&canister_id, &wasm_module)
                .with_raw_arg(args.to_vec())
                .with_mode(mode);
            let install_result = install_builder
                .build()
                .context("Failed to build InstallCode call.")?
                .await;
            if install_result.is_ok() {
                start_canister(env, canister_id, &CallSender::SelectedId).await?;
                let status =
                    canister::get_canister_status(env, canister_id, &CallSender::SelectedId)
                        .await?;
                let cycles = status.cycles.0.to_u128().unwrap();
                let mut attempts = 0_u128;
                loop {
                    let margin =
                        initial_margin.unwrap_or(WITHDRAWAL_COST) + attempts * WITHDRAWAL_COST / 10;
                    if margin >= cycles {
                        info!(log, "Too few cycles to withdraw: {}.", cycles);
                        break;
                    }
                    let cycles_to_withdraw = cycles - margin;
                    debug!(
                        log,
                        "Margin: {margin}. Withdrawing {cycles_to_withdraw} cycles."
                    );
                    let result = match withdraw_target {
                        WithdrawTarget::NoWithdrawal => Ok(()),
                        WithdrawTarget::Dank => {
                            info!(
                                log,
                                "Attempting to transfer {} cycles to dank principal {}.",
                                cycles_to_withdraw,
                                dank_target_principal
                            );
                            let wallet = build_wallet_canister(canister_id, agent).await?;
                            let opt_principal = Some(dank_target_principal);
                            wallet
                                .call(
                                    DANK_PRINCIPAL,
                                    "mint",
                                    Argument::from_candid((opt_principal,)),
                                    cycles_to_withdraw,
                                )
                                .await
                                .context("Failed mint call.")
                        }
                        WithdrawTarget::Canister {
                            canister_id: target_canister_id,
                        } => {
                            info!(
                                log,
                                "Attempting to transfer {} cycles to canister {}.",
                                cycles_to_withdraw,
                                target_canister_id
                            );
                            // Transfer cycles from the source canister to the target canister using the temporary wallet.
                            deposit_cycles(
                                env,
                                target_canister_id,
                                &CallSender::Wallet(canister_id),
                                cycles_to_withdraw,
                            )
                            .await
                        }
                        WithdrawTarget::CyclesLedger { to } => {
                            wallet_deposit_to_cycles_ledger(
                                agent,
                                canister_id,
                                cycles_to_withdraw,
                                to,
                            )
                            .await
                        }
                    };
                    if result.is_ok() {
                        info!(log, "Successfully withdrew {} cycles.", cycles_to_withdraw);
                        break;
                    } else {
                        let message = format!("{:?}", result);
                        if message.contains("Couldn't send message")
                            || message.contains("out of cycles")
                        {
                            info!(log, "Not enough margin. Trying again with more margin.");
                            attempts += 1;
                        } else {
                            // Unforeseen error. Report it back to user
                            result?;
                        }
                    }
                }
                stop_canister(env, canister_id, &CallSender::SelectedId).await?;
            } else {
                info!(
                    log,
                    "Failed to install temporary wallet, deleting without withdrawal."
                );
                if status.status != CanisterStatus::Stopped {
                    info!(log, "Stopping canister.")
                }
                stop_canister(env, canister_id, &CallSender::SelectedId).await?;
            }
            call_sender = &CallSender::SelectedId;
        }

        info!(
            log,
            "Deleting canister {}, with canister_id {}",
            canister,
            canister_id.to_text(),
        );

        canister::delete_canister(env, canister_id, call_sender).await?;
    }

    if let Some(canister_name) = canister_name_to_delete {
        canister_id_store.remove(&canister_name)?;
    }

    Ok(())
}

pub async fn exec(
    env: &dyn Environment,
    opts: CanisterDeleteOpts,
    call_sender: &CallSender,
) -> DfxResult {
    let config = env.get_config_or_anyhow()?;

    fetch_root_key_if_needed(env).await?;

    if let Some(canister) = opts.canister.as_deref() {
        delete_canister(
            env,
            canister,
            call_sender,
            opts.no_withdrawal,
            opts.yes,
            opts.withdraw_cycles_to_canister,
            opts.withdraw_cycles_to_dank,
            opts.withdraw_cycles_to_dank_principal,
            opts.to_subaccount,
            opts.initial_margin,
        )
        .await
    } else if opts.all {
        if let Some(canisters) = &config.get_config().canisters {
            for canister in canisters.keys() {
                delete_canister(
                    env,
                    canister,
                    call_sender,
                    opts.no_withdrawal,
                    opts.yes,
                    opts.withdraw_cycles_to_canister.clone(),
                    opts.withdraw_cycles_to_dank,
                    opts.withdraw_cycles_to_dank_principal.clone(),
                    opts.to_subaccount,
                    opts.initial_margin,
                )
                .await?;
            }
        }
        Ok(())
    } else {
        unreachable!()
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum WithdrawTarget {
    NoWithdrawal,
    Dank,
    CyclesLedger { to: Account },
    Canister { canister_id: Principal },
}
