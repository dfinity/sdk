use crate::commands::ledger::{get_icpts_from_args, transfer};
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ledger_types::{Memo, MAINNET_LEDGER_CANISTER_ID};
use crate::lib::nns_types::account_identifier::{AccountIdentifier, Subaccount};
use crate::lib::nns_types::icpts::{ICPTs, TRANSACTION_FEE};
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::clap::validators::{e8s_validator, icpts_amount_validator, memo_validator};

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
    #[clap(long)]
    from_subaccount: Option<Subaccount>,

    /// ICPs to transfer to the destination AccountIdentifier
    /// Can be specified as a Decimal with the fractional portion up to 8 decimal places
    /// i.e. 100.012
    #[clap(long, validator(icpts_amount_validator))]
    amount: Option<String>,

    /// Specify ICP as a whole number, helpful for use in conjunction with `--e8s`
    #[clap(long, validator(e8s_validator), conflicts_with("amount"))]
    icp: Option<String>,

    /// Specify e8s as a whole number, helpful for use in conjunction with `--icp`
    #[clap(long, validator(e8s_validator), conflicts_with("amount"))]
    e8s: Option<String>,

    /// Specify a numeric memo for this transaction.
    #[clap(long, validator(memo_validator))]
    memo: String,

    /// Transaction fee, default is 10000 e8s.
    #[clap(long, validator(icpts_amount_validator))]
    fee: Option<String>,

    #[clap(long)]
    /// Canister ID of the ledger canister.
    ledger_canister_id: Option<Principal>,
}

pub async fn exec(env: &dyn Environment, opts: TransferOpts) -> DfxResult {
    let amount = get_icpts_from_args(&opts.amount, &opts.icp, &opts.e8s)?;

    let fee = opts.fee.clone().map_or(TRANSACTION_FEE, |v| {
        ICPTs::from_str(&v).expect("bug: amount_validator did not validate the fee")
    });

    let memo = Memo(
        opts.memo
            .parse::<u64>()
            .expect("bug: memo_validator did not validate the memo"),
    );

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

    let block_height = transfer(
        agent,
        &canister_id,
        memo,
        amount,
        fee,
        opts.from_subaccount,
        to,
    )
    .await?;

    println!("Transfer sent at BlockHeight: {}", block_height);

    Ok(())
}
