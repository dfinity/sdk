use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::nns_types::account_identifier::{AccountIdentifier, Subaccount};
use crate::lib::nns_types::icpts::{ICPTs, TRANSACTION_FEE};
use crate::lib::nns_types::{
    BlockHeight, CreateCanisterResult, Memo, NotifyCanisterArgs, SendArgs,
    TransactionNotificationResult, CYCLE_MINTER_CANISTER_ID, LEDGER_CANISTER_ID,
};
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::waiter::waiter_with_timeout;
use crate::util::clap::validators::icpts_amount_validator;
use crate::util::expiry_duration;

use anyhow::anyhow;
use candid::{Decode, Encode};
use clap::{ArgSettings, Clap};
use ic_types::principal::Principal;
use std::str::FromStr;

const SEND_METHOD: &str = "send_dfx";
const NOTIFY_METHOD: &str = "notify_dfx";
const MEMO_CREATE_CANISTER: u64 = 1095062083_u64;

/// Create a canister from ICP
#[derive(Clap)]
pub struct CreateCanisterOpts {
    /// ICP to account for the fee and the rest to mint as a cycle deposit
    #[clap(long, validator(icpts_amount_validator))]
    amount: String,

    /// Transaction fee, default is 137 Doms.
    #[clap(long, validator(icpts_amount_validator), setting = ArgSettings::Hidden)]
    fee: Option<String>,

    /// Specify the controller of the new canister
    #[clap(long)]
    controller: String,

    /// Max fee
    #[clap(long, validator(icpts_amount_validator), setting = ArgSettings::Hidden)]
    max_fee: Option<String>,
}

pub async fn exec(env: &dyn Environment, opts: CreateCanisterOpts) -> DfxResult {
    let amount = ICPTs::from_str(&opts.amount).map_err(|err| anyhow!(err))?;

    let fee = opts.fee.map_or(Ok(TRANSACTION_FEE), |v| {
        ICPTs::from_str(&v).map_err(|err| anyhow!(err))
    })?;

    // validated by memo_validator
    let memo = Memo(MEMO_CREATE_CANISTER);

    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    fetch_root_key_if_needed(env).await?;

    let ledger_canister_id = Principal::from_text(LEDGER_CANISTER_ID)?;

    let cycle_minter_id = Principal::from_text(CYCLE_MINTER_CANISTER_ID)?;

    let to_subaccount = Some(Subaccount::from(&Principal::from_text(opts.controller)?));
    let to = AccountIdentifier::new(cycle_minter_id.clone(), to_subaccount);

    let result = agent
        .update(&ledger_canister_id, SEND_METHOD)
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

    let max_fee = opts
        .max_fee
        .map_or(ICPTs::new(0, 0).map_err(|err| anyhow!(err)), |v| {
            ICPTs::from_str(&v).map_err(|err| anyhow!(err))
        })?;

    let result = agent
        .update(&ledger_canister_id, NOTIFY_METHOD)
        .with_arg(Encode!(&NotifyCanisterArgs {
            block_height,
            max_fee,
            from_subaccount: None,
            to_canister: cycle_minter_id,
            to_subaccount,
        })?)
        .call_and_wait(waiter_with_timeout(expiry_duration()))
        .await?;

    let result = Decode!(&result, TransactionNotificationResult)?;

    let result = Decode!(&result.0, CreateCanisterResult)?;

    match result {
        Ok(v) => {
            println!("Canister created with id: {:?}", v.to_text());
        }
        Err((msg, maybe_height)) => {
            println!("Error: {}\nMaybe BlockHeight:{:?}", msg, maybe_height);
        }
    };
    Ok(())
}
