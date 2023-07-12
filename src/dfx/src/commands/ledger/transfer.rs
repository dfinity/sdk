use crate::commands::ledger::get_icpts_from_args;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ledger_types::{Memo, MAINNET_LEDGER_CANISTER_ID};
use crate::lib::nns_types::account_identifier::{AccountIdentifier, Subaccount};
use crate::lib::nns_types::icpts::{ICPTs, TRANSACTION_FEE};
use crate::lib::operations::ledger::transfer;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::clap::parsers::{e8s_parser, memo_parser};

use anyhow::{anyhow, Context};
use candid::Principal;
use clap::Parser;
use std::str::FromStr;

/// Transfer ICP from the user to the destination account identifier.
#[derive(Parser)]
pub struct TransferOpts {
    /// AccountIdentifier of transfer destination.
    to: String,

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

    #[arg(long)]
    /// Canister ID of the ledger canister.
    ledger_canister_id: Option<Principal>,

    /// Transaction timestamp, in nanoseconds, for use in controlling transaction-deduplication, default is system-time. // https://internetcomputer.org/docs/current/developer-docs/integrations/icrc-1/#transaction-deduplication-
    #[arg(long)]
    created_at_time: Option<u64>,
}

pub async fn exec(env: &dyn Environment, opts: TransferOpts) -> DfxResult {
    let amount = get_icpts_from_args(opts.amount, opts.icp, opts.e8s)?;

    let fee = opts.fee.unwrap_or(TRANSACTION_FEE);

    let memo = Memo(opts.memo);

    let to = AccountIdentifier::from_str(&opts.to)
        .map_err(|e| anyhow!(e))
        .with_context(|| {
            format!(
                "Failed to parse transfer destination from string '{}'.",
                &opts.to
            )
        })?
        .to_address();

    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    fetch_root_key_if_needed(env).await?;

    let canister_id = opts
        .ledger_canister_id
        .unwrap_or(MAINNET_LEDGER_CANISTER_ID);

    let _block_height = transfer(
        agent,
        &canister_id,
        memo,
        amount,
        fee,
        opts.from_subaccount,
        to,
        opts.created_at_time,
    )
    .await?;

    Ok(())
}
