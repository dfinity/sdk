use crate::lib::error::DfxResult;
use crate::lib::identity::identity_utils::CallSender;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::operations::canister;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::{environment::Environment, identity::Identity};
use crate::util::clap::validators::cycle_amount_validator;
use crate::util::expiry_duration;

use anyhow::Context;
use candid::Principal;
use clap::Parser;
use slog::info;
use std::time::Duration;

/// Deposit cycles into the specified canister.
#[derive(Parser)]
pub struct DepositCyclesOpts {
    /// Specifies the amount of cycles to send on the call.
    /// Deducted from the wallet.
    #[clap(validator(cycle_amount_validator))]
    cycles: String,

    /// Specifies the name or id of the canister to receive the cycles deposit.
    /// You must specify either a canister name/id or the --all option.
    canister: Option<String>,

    /// Deposit cycles to all of the canisters configured in the dfx.json file.
    #[clap(long, required_unless_present("canister"))]
    all: bool,
}

async fn deposit_cycles(
    env: &dyn Environment,
    canister: &str,
    timeout: Duration,
    call_sender: &CallSender,
    cycles: u128,
) -> DfxResult {
    let log = env.get_logger();
    let canister_id_store = CanisterIdStore::for_env(env)?;
    let canister_id =
        Principal::from_text(canister).or_else(|_| canister_id_store.get(canister))?;

    info!(log, "Depositing {} cycles onto {}", cycles, canister,);

    canister::deposit_cycles(env, canister_id, timeout, call_sender, cycles).await?;

    let status = canister::get_canister_status(env, canister_id, timeout, call_sender).await;
    if let Ok(status) = status {
        info!(
            log,
            "Deposited {} cycles, updated balance: {} cycles", cycles, status.cycles
        );
    } else {
        info!(log, "Deposited {cycles} cycles.");
    }

    Ok(())
}

pub async fn exec(
    env: &dyn Environment,
    opts: DepositCyclesOpts,
    mut call_sender: &CallSender,
) -> DfxResult {
    let proxy_sender;

    // choose default wallet if no wallet is specified
    if call_sender == &CallSender::SelectedId {
        let wallet = Identity::get_or_create_wallet_canister(
            env,
            env.get_network_descriptor(),
            env.get_selected_identity().expect("No selected identity"),
        )
        .await?;
        proxy_sender = CallSender::Wallet(*wallet.canister_id_());
        call_sender = &proxy_sender;
    }

    // amount has been validated by cycle_amount_validator
    let cycles = opts.cycles.parse::<u128>().unwrap();

    let config = env.get_config_or_anyhow()?;

    fetch_root_key_if_needed(env).await?;
    let timeout = expiry_duration();

    if let Some(canister) = opts.canister.as_deref() {
        deposit_cycles(env, canister, timeout, call_sender, cycles).await
    } else if opts.all {
        if let Some(canisters) = &config.get_config().canisters {
            for canister in canisters.keys() {
                deposit_cycles(env, canister, timeout, call_sender, cycles)
                    .await
                    .with_context(|| format!("Failed to deposit cycles into {}.", canister))?;
            }
        }
        Ok(())
    } else {
        unreachable!()
    }
}
