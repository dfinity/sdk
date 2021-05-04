use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::nns_types::account_identifier::AccountIdentifier;
use crate::lib::nns_types::icpts::{ICPTs, TRANSACTION_FEE};
use crate::lib::nns_types::{BlockHeight, Memo, SendArgs, LEDGER_CANISTER_ID};
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::waiter::waiter_with_timeout;
use crate::util::clap::validators::{icpts_amount_validator, memo_validator};
use crate::util::expiry_duration;

use anyhow::anyhow;
use candid::{Decode, Encode};
use clap::{ArgSettings, Clap};
use ic_types::principal::Principal;
use std::str::FromStr;

const SEND_METHOD: &str = "send_dfx";

/// Transfer ICP from the user to the destination AccountIdentifier
#[derive(Clap)]
pub struct TransferOpts {
    /// ICPs to transfer
    #[clap(long, validator(icpts_amount_validator))]
    amount: String,

    /// Specify a numeric memo for this transaction.
    #[clap(long, validator(memo_validator))]
    memo: String,

    /// Transaction fee, default is 137 Doms.
    #[clap(long, validator(icpts_amount_validator), setting = ArgSettings::Hidden)]
    fee: Option<String>,

    /// AccountIdentifier of transfer destination.
    #[clap(long)]
    to: String,
}

pub async fn exec(env: &dyn Environment, opts: TransferOpts) -> DfxResult {
    let amount = ICPTs::from_str(&opts.amount).map_err(|err| anyhow!(err))?;

    let fee = opts.fee.map_or(Ok(TRANSACTION_FEE), |v| {
        ICPTs::from_str(&v).map_err(|err| anyhow!(err))
    })?;

    // validated by memo_validator
    let memo = Memo(opts.memo.parse::<u64>().unwrap());

    let to = AccountIdentifier::from_str(&opts.to).map_err(|err| anyhow!(err))?;

    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    fetch_root_key_if_needed(env).await?;

    let canister_id = Principal::from_text(LEDGER_CANISTER_ID)?;

    let result = agent
        .update(&canister_id, SEND_METHOD)
        .with_arg(Encode!(&SendArgs {
            memo,
            amount,
            fee,
            from_subaccount: None,
            to,
            created_at_time: None,
        })?)
        .call_and_wait(waiter_with_timeout(expiry_duration()))
        .await?;

    let block_height = Decode!(&result, BlockHeight)?;
    println!("Transfer sent at BlockHeight: {}", block_height);

    Ok(())
}
