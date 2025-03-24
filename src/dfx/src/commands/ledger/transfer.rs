use crate::commands::ledger::get_icpts_from_args;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ledger_types::{Memo, MAINNET_LEDGER_CANISTER_ID};
use crate::lib::nns_types::account_identifier::{AccountIdentifier, Subaccount};
use crate::lib::nns_types::icpts::{ICPTs, TRANSACTION_FEE};
use crate::lib::operations::ledger::{icrc1_transfer, transfer};
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::clap::parsers::{e8s_parser, icrc_subaccount_parser, memo_parser};
use anyhow::{anyhow, Context};
use candid::Principal;
use clap::Parser;
use icrc_ledger_types::icrc1;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

/// Transfer ICP from the user to the destination account identifier or principal.
#[derive(Parser)]
pub struct TransferOpts {
    /// AccountIdentifier of transfer destination.
    to: Option<String>,

    /// Principal of transfer destination.
    #[arg(long, conflicts_with("to"))]
    to_principal: Option<Principal>,

    /// Transfer cycles to this subaccount.
    #[arg(long, value_parser = icrc_subaccount_parser, requires("to_principal"))]
    to_subaccount: Option<icrc1::account::Subaccount>,

    /// Subaccount to transfer from.
    #[arg(long)]
    from_subaccount: Option<Subaccount>,

    /// ICPs to transfer to the destination AccountIdentifier
    /// Can be specified as a Decimal with the fractional portion up to 8 decimal places
    /// i.e. 100.012
    #[arg(long)]
    amount: Option<ICPTs>,

    /// Specify ICP as a whole number, helpful for use in conjunction with `--e8s`
    #[arg(long, value_parser = e8s_parser, conflicts_with("amount"))]
    icp: Option<u64>,

    /// Specify e8s as a whole number, helpful for use in conjunction with `--icp`
    #[arg(long, value_parser = e8s_parser, conflicts_with("amount"))]
    e8s: Option<u64>,

    /// Specify a numeric memo for this transaction.
    #[arg(long, value_parser = memo_parser)]
    memo: u64,

    /// Transaction fee, default is 10000 e8s.
    #[arg(long)]
    fee: Option<ICPTs>,

    /// Canister ID of the ledger canister.
    #[arg(long)]
    ledger_canister_id: Option<Principal>,

    /// Transaction timestamp, in nanoseconds, for use in controlling transaction-deduplication, default is system-time. // https://internetcomputer.org/docs/current/developer-docs/integrations/icrc-1/#transaction-deduplication-
    #[arg(long)]
    created_at_time: Option<u64>,
}

pub async fn exec(env: &dyn Environment, opts: TransferOpts) -> DfxResult {
    let amount = get_icpts_from_args(opts.amount, opts.icp, opts.e8s)?;

    let agent = env.get_agent();

    fetch_root_key_if_needed(env).await?;

    let canister_id = opts
        .ledger_canister_id
        .unwrap_or(MAINNET_LEDGER_CANISTER_ID);

    if let Some(to) = opts.to {
        let fee = opts.fee.unwrap_or(TRANSACTION_FEE);
        let memo = Memo(opts.memo);

        let to = AccountIdentifier::from_str(&to)
            .map_err(|e| anyhow!(e))
            .with_context(|| {
                format!(
                    "Failed to parse transfer destination from string '{}'.",
                    &to
                )
            })?
            .to_address();

        let _block_height = transfer(
            agent,
            env.get_logger(),
            &canister_id,
            memo,
            amount,
            fee,
            opts.from_subaccount,
            to,
            opts.created_at_time,
        )
        .await?;
    } else if let Some(to) = opts.to_principal {
        let to = icrc1::account::Account {
            owner: to,
            subaccount: opts.to_subaccount,
        };

        let created_at_time = opts.created_at_time.unwrap_or(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
        );

        let result = icrc1_transfer(
            agent,
            env.get_logger(),
            &canister_id,
            opts.from_subaccount.map(|s| s.0),
            to,
            amount,
            opts.fee,
            Some(opts.memo),
            created_at_time,
        )
        .await;

        if result.is_err() && opts.created_at_time.is_none() {
            slog::warn!(
                env.get_logger(),
                "If you retry this operation, use --created-at-time {}",
                created_at_time
            );
        }
        let block_index = result?;

        slog::info!(
            env.get_logger(),
            "Transfer sent at block index {}",
            block_index
        );
    } else {
        return Err(anyhow!("Please provide the transfer destination."));
    }

    Ok(())
}
