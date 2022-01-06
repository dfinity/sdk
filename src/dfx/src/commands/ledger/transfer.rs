use crate::commands::ledger::{get_icpts_from_args, retryable};
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ledger_types::{
    TransferArgs, TransferError, TransferResult, MAINNET_LEDGER_CANISTER_ID,
};
use crate::lib::nns_types::account_identifier::AccountIdentifier;
use crate::lib::nns_types::icpts::{ICPTs, TRANSACTION_FEE};
use crate::lib::nns_types::{BlockHeight, Memo, SendArgs, TimeStamp, LEDGER_CANISTER_ID};
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::waiter::waiter_with_timeout;
use crate::util::clap::validators::{e8s_validator, icpts_amount_validator, memo_validator};
use crate::util::expiry_duration;

use anyhow::{anyhow, bail};
use candid::{Decode, Encode};
use clap::Clap;
use ic_types::principal::Principal;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use garcon::{Delay, Waiter};

const TRANSFER_METHOD: &str = "transfer";

/// Transfer ICP from the user to the destination AccountIdentifier
#[derive(Clap)]
pub struct TransferOpts {
    /// AccountIdentifier of transfer destination.
    to: String,

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
}

pub async fn exec(env: &dyn Environment, opts: TransferOpts) -> DfxResult {
    let amount = get_icpts_from_args(opts.amount, opts.icp, opts.e8s)?;

    let fee = opts.fee.map_or(Ok(TRANSACTION_FEE), |v| {
        ICPTs::from_str(&v).map_err(|err| anyhow!(err))
    })?;

    // validated by memo_validator
    let memo = Memo(opts.memo.parse::<u64>().unwrap());

    let to = AccountIdentifier::from_str(&opts.to).map_err(|err| anyhow!(err))?.to_address();
    let timestamp_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;

    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    fetch_root_key_if_needed(env).await?;

    let canister_id = Principal::from_text(LEDGER_CANISTER_ID)?;

    let mut waiter = Delay::builder()
        .with(Delay::count_timeout(30))
        .exponential_backoff_capped(
            std::time::Duration::from_secs(1),
            2.0,
            std::time::Duration::from_secs(16),
        )
        .build();
    waiter.start();

    let mut n = 0;

    let block_height = loop {
        match agent
            .update(&MAINNET_LEDGER_CANISTER_ID, TRANSFER_METHOD)
            .with_arg(Encode!(&TransferArgs {
            memo,
            amount,
            fee,
            from_subaccount: None,
            to,
            created_at_time: Some(TimeStamp { timestamp_nanos }),
        })?)
            .call_and_wait(waiter_with_timeout(expiry_duration()))
            .await {
            Ok(result) => {
                let transfer_result = Decode!(&result, TransferResult)?;
                eprintln!("transfer result: {:?}", &result);
                n = n + 1;
                if n < 2 {
                    if let Ok(_) = waiter.async_wait().await {
                        eprintln!("force retry (no error)");
                        continue;
                    }
                }
                match transfer_result {
                    Ok(block_height) => break block_height,
                    Err(TransferError::TxDuplicate { duplicate_of }) => break duplicate_of,
                    Err(transfer_err) => bail!(transfer_err),
                }
            }
            Err(agent_err) if !retryable(&agent_err) => {
                eprintln!("non-retryable error");
                bail!(agent_err);
            }
            Err(agent_err) => {
                eprintln!("retryable error {:?}", &agent_err);
                if let Err(_waiter_err) = waiter.async_wait().await {
                    bail!(agent_err);
                }
            }
        }
    };
    println!("Transfer sent at BlockHeight: {}", block_height);

    Ok(())
}
