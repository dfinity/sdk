use crate::commands::ledger::{get_icpts_from_args, send_and_notify};
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::nns_types::account_identifier::Subaccount;
use crate::lib::nns_types::icpts::{ICPTs, TRANSACTION_FEE};
use crate::lib::nns_types::{CyclesResponse, Memo};

use crate::util::clap::validators::{e8s_validator, icpts_amount_validator};

use anyhow::anyhow;
use clap::Clap;
use ic_types::principal::Principal;
use std::str::FromStr;

const MEMO_TOP_UP_CANISTER: u64 = 1347768404_u64;

/// Top up a canister with cycles minted from ICP
#[derive(Clap)]
pub struct TopUpOpts {
    /// Specify the canister id to top up
    canister: String,

    /// ICP to mint into cycles and deposit into destination canister
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

    /// Transaction fee, default is 10000 e8s.
    #[clap(long, validator(icpts_amount_validator))]
    fee: Option<String>,

    /// Max fee, default is 10000 e8s.
    #[clap(long, validator(icpts_amount_validator))]
    max_fee: Option<String>,
}

pub async fn exec(env: &dyn Environment, opts: TopUpOpts) -> DfxResult {
    let amount = get_icpts_from_args(opts.amount, opts.icp, opts.e8s)?;

    let fee = opts.fee.map_or(Ok(TRANSACTION_FEE), |v| {
        ICPTs::from_str(&v).map_err(|err| anyhow!(err))
    })?;

    let memo = Memo(MEMO_TOP_UP_CANISTER);

    let to_subaccount = Some(Subaccount::from(&Principal::from_text(opts.canister)?));

    let max_fee = opts
        .max_fee
        .map_or(Ok(TRANSACTION_FEE), |v| ICPTs::from_str(&v))
        .map_err(|err| anyhow!(err))?;

    let result = send_and_notify(env, memo, amount, fee, to_subaccount, max_fee).await?;

    match result {
        CyclesResponse::ToppedUp(()) => {
            println!("Canister was topped up!");
        }
        CyclesResponse::Refunded(msg, maybe_block_height) => {
            match maybe_block_height {
                Some(height) => {
                    println!("Refunded at block height {} with message :{}", height, msg)
                }
                None => println!("Refunded with message: {}", msg),
            };
        }
        CyclesResponse::CanisterCreated(_) => unreachable!(),
    };
    Ok(())
}
