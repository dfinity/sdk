use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_utils::CallSender;
use crate::lib::identity::Identity;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::operations::canister;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::waiter::waiter_with_timeout;
use crate::util::assets::wallet_wasm;
use crate::util::{blob_from_arguments, expiry_duration};
use candid::CandidType;
use ic_utils::call::AsyncCall;

use anyhow::{anyhow, bail};
use clap::Clap;
use ic_types::Principal;
use ic_utils::interfaces::management_canister::builders::{CanisterInstall, InstallMode};
use ic_utils::interfaces::ManagementCanister;
use num_traits::cast::ToPrimitive;
use slog::info;
use std::time::Duration;

const WITHDRAWL_COST: u64 = 3_000_000_000; // Emperically ~ 2B.

/// Deletes a canister on the Internet Computer network.
#[derive(Clap)]
pub struct CanisterDeleteOpts {
    /// Specifies the name of the canister to delete.
    /// You must specify either a canister name/id or the --all flag.
    canister: Option<String>,

    /// Deletes all of the canisters configured in the dfx.json file.
    #[clap(long, required_unless_present("canister"))]
    all: bool,

    /// Withdraw cycles from canister(s) to wallet before deleting.
    #[clap(long)]
    withdraw_cycles: bool,
}

async fn delete_canister(
    env: &dyn Environment,
    canister: &str,
    timeout: Duration,
    call_sender: &CallSender,
    withdraw_cycles: bool,
) -> DfxResult {
    let log = env.get_logger();
    let mut canister_id_store = CanisterIdStore::for_env(env)?;
    let canister_id =
        Principal::from_text(canister).or_else(|_| canister_id_store.get(canister))?;
    if withdraw_cycles {
        // Get the wallet to transfer the cycles to.
        let target_wallet_canister_id = match call_sender {
            CallSender::SelectedId => {
                bail!("no target wallet given for cycles.");
            }
            CallSender::Wallet(wallet_id) | CallSender::SelectedIdWallet(wallet_id) => *wallet_id,
        };
        fetch_root_key_if_needed(env).await?;

        // Determine how many cycles we can withdraw.
        let status = canister::get_canister_status(env, canister_id, timeout, call_sender).await?;
        let mut cycles = status.cycles.0.to_u64().unwrap();
        if cycles > WITHDRAWL_COST {
            cycles = cycles - WITHDRAWL_COST;
            info!(
                log,
                "Beginning withdrawl of {} cycles to wallet {}.", cycles, target_wallet_canister_id
            );

            // Install a temporary wallet wasm which will transfer the cycles out of the canister before it is deleted.
            let wasm_module = wallet_wasm(env.get_logger())?;
            let agent = env
                .get_agent()
                .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;
            let mgr = ManagementCanister::create(agent);
            let canister_id =
                Principal::from_text(canister).or_else(|_| canister_id_store.get(canister))?;
            info!(
                log,
                "Installing temporary wasm in canister {} to enable transfer of cycles.", canister
            );
            let args = blob_from_arguments(None, None, None, &None)?;
            let mode = InstallMode::Reinstall;
            match call_sender {
                CallSender::SelectedId => {
                    let install_builder = mgr
                        .install_code(&canister_id, &wasm_module)
                        .with_raw_arg(args.to_vec())
                        .with_mode(mode);
                    install_builder
                        .build()?
                        .call_and_wait(waiter_with_timeout(timeout))
                        .await?;
                }
                CallSender::Wallet(wallet_id) | CallSender::SelectedIdWallet(wallet_id) => {
                    let wallet = Identity::build_wallet_canister(*wallet_id, env)?;
                    let install_args = CanisterInstall {
                        mode,
                        canister_id,
                        wasm_module,
                        arg: args.to_vec(),
                    };
                    wallet
                        .call_forward(
                            mgr.update_("install_code").with_arg(install_args).build(),
                            0,
                        )?
                        .call_and_wait(waiter_with_timeout(timeout))
                        .await?;
                }
            }

            // Transfer cycles from the canister to the regular wallet using the temporary wallet.
            #[derive(CandidType)]
            struct In {
                canister: Principal,
                amount: u64,
            }
            let source_wallet = Identity::build_wallet_canister(canister_id, env)?;
            info!(log, "Transfering cycles.");
            let withdraw_args = In {
                canister: target_wallet_canister_id,
                amount: cycles,
            };
            match call_sender {
                CallSender::SelectedId => {
                    source_wallet
                        .update_("wallet_send")
                        .with_arg(withdraw_args)
                        .build()
                        .call_and_wait(waiter_with_timeout(expiry_duration()))
                        .await?;
                }
                CallSender::Wallet(wallet_id) | CallSender::SelectedIdWallet(wallet_id) => {
                    let wallet = Identity::build_wallet_canister(*wallet_id, env)?;
                    wallet
                        .call_forward(
                            source_wallet
                                .update_("wallet_send")
                                .with_arg(withdraw_args)
                                .build(),
                            0,
                        )?
                        .call_and_wait(waiter_with_timeout(timeout))
                        .await?;
                }
            }
            info!(log, "Transfer successful.");
        } else {
            info!(log, "Too few cycles to withdraw: {}.", cycles);
        }
    }

    info!(
        log,
        "Deleting code for canister {}, with canister_id {}",
        canister,
        canister_id.to_text(),
    );

    canister::delete_canister(env, canister_id, timeout, &call_sender).await?;

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
        delete_canister(env, canister, timeout, call_sender, opts.withdraw_cycles).await
    } else if opts.all {
        if let Some(canisters) = &config.get_config().canisters {
            for canister in canisters.keys() {
                delete_canister(env, canister, timeout, call_sender, opts.withdraw_cycles).await?;
            }
        }
        Ok(())
    } else {
        unreachable!()
    }
}
