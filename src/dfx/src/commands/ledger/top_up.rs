use crate::commands::ledger::{get_icpts_from_args, notify_top_up, transfer_cmc};
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ledger_types::{Memo, NotifyError};
use crate::lib::nns_types::account_identifier::Subaccount;
use crate::lib::nns_types::icpts::{ICPTs, TRANSACTION_FEE};

use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::clap::validators::{e8s_validator, icpts_amount_validator};

use anyhow::{anyhow, bail, Context};
use candid::Principal;
use clap::Parser;
use std::str::FromStr;

const MEMO_TOP_UP_CANISTER: u64 = 1347768404_u64;

/// Top up a canister with cycles minted from ICP
#[derive(Parser)]
pub struct TopUpOpts {
    /// Specify the canister id to top up
    canister: String,

    /// Subaccount to withdraw from
    #[clap(long)]
    from_subaccount: Option<Subaccount>,

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
    let amount = get_icpts_from_args(&opts.amount, &opts.icp, &opts.e8s)?;

    let fee = opts
        .fee
        .as_ref()
        .map_or(Ok(TRANSACTION_FEE), |v| {
            ICPTs::from_str(v).map_err(|err| anyhow!(err))
        })
        .context("Failed to determine fee.")?;

    let memo = Memo(MEMO_TOP_UP_CANISTER);

    let to = Principal::from_text(&opts.canister).with_context(|| {
        format!(
            "Failed to parse {:?} as target canister principal.",
            &opts.canister
        )
    })?;

    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    fetch_root_key_if_needed(env).await?;

    let height = transfer_cmc(agent, memo, amount, fee, opts.from_subaccount, to).await?;
    println!("Transfer sent at block height {height}");
    let result = notify_top_up(agent, to, height).await?;

    match result {
        Ok(cycles) => {
            println!("Canister was topped up with {cycles} cycles!");
        }
        Err(NotifyError::Refunded {
            reason,
            block_index,
        }) => match block_index {
            Some(height) => {
                println!("Refunded at block height {height} with message: {reason}")
            }
            None => println!("Refunded with message: {reason}"),
        },
        Err(other) => bail!("{other:?}"),
    };
    Ok(())
}
