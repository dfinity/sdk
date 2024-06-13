use std::time::{SystemTime, UNIX_EPOCH};

use crate::lib::error::DfxResult;
use crate::lib::identity::wallet::get_or_create_wallet_canister;
use crate::lib::operations::canister;
use crate::lib::operations::cycles_ledger::cycles_ledger_enabled;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::{environment::Environment, operations::cycles_ledger};
use crate::util::clap::parsers::{cycle_amount_parser, icrc_subaccount_parser};
use anyhow::{bail, Context};
use candid::Principal;
use clap::Parser;
use dfx_core::identity::CallSender;
use icrc_ledger_types::icrc1::account::Subaccount;
use slog::{debug, info, warn};

/// Deposit cycles into the specified canister.
#[derive(Parser)]
pub struct DepositCyclesOpts {
    /// Specifies the amount of cycles to send on the call.
    /// Deducted from the wallet.
    #[arg(value_parser = cycle_amount_parser)]
    cycles: u128,

    /// Specifies the name or id of the canister to receive the cycles deposit.
    /// You must specify either a canister name/id or the --all option.
    canister: Option<String>,

    /// Deposit cycles to all of the canisters configured in the dfx.json file.
    #[arg(long, required_unless_present("canister"))]
    all: bool,

    /// Use cycles from this subaccount.
    #[arg(long, value_parser = icrc_subaccount_parser)]
    from_subaccount: Option<Subaccount>,

    /// Transaction timestamp, in nanoseconds, for use in controlling transaction deduplication, default is system time.
    /// https://internetcomputer.org/docs/current/developer-docs/integrations/icrc-1/#transaction-deduplication-
    #[arg(long)]
    created_at_time: Option<u64>,
}

async fn deposit_cycles(
    env: &dyn Environment,
    canister: &str,
    call_sender: &CallSender,
    cycles: u128,
    created_at_time: u64,
    from_subaccount: Option<Subaccount>,
) -> DfxResult {
    let log = env.get_logger();
    let canister_id_store = env.get_canister_id_store()?;
    let canister_id =
        Principal::from_text(canister).or_else(|_| canister_id_store.get(canister))?;

    info!(log, "Depositing {} cycles onto {}", cycles, canister,);

    match call_sender {
        CallSender::SelectedId => {
            if !cycles_ledger_enabled() {
                // should be unreachable
                bail!("No wallet configured");
            }
            cycles_ledger::withdraw(
                env.get_agent(),
                env.get_logger(),
                canister_id,
                cycles,
                created_at_time,
                from_subaccount,
            )
            .await?;
        }
        CallSender::Wallet(_) => {
            canister::deposit_cycles(env, canister_id, call_sender, cycles).await?
        }
    };

    let status = canister::get_canister_status(env, canister_id, call_sender).await;
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
    fetch_root_key_if_needed(env).await?;

    let proxy_sender;

    if call_sender == &CallSender::SelectedId {
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
                if cycles_ledger_enabled() && matches!(err, crate::lib::identity::wallet::GetOrCreateWalletCanisterError::NoWalletConfigured { .. }) {
                    debug!(env.get_logger(), "No wallet configured");
                } else {
                    bail!(err)
                }
            },
        }
    }

    let created_at_time = opts.created_at_time.unwrap_or_else(|| {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        if matches!(call_sender, CallSender::SelectedId) {
            warn!(
                env.get_logger(),
                "If you retry this operation, use --created-at-time {}", now
            );
        }
        now
    });

    // amount has been validated by cycle_amount_validator
    let cycles = opts.cycles;

    if let Some(canister) = opts.canister.as_deref() {
        deposit_cycles(
            env,
            canister,
            call_sender,
            cycles,
            created_at_time,
            opts.from_subaccount,
        )
        .await
    } else if opts.all {
        let config = env.get_config_or_anyhow()?;

        if let Some(canisters) = &config.get_config().canisters {
            for canister in canisters.keys() {
                deposit_cycles(
                    env,
                    canister,
                    call_sender,
                    cycles,
                    created_at_time,
                    opts.from_subaccount,
                )
                .await
                .with_context(|| format!("Failed to deposit cycles into {}.", canister))?;
            }
        }
        Ok(())
    } else {
        unreachable!()
    }
}
