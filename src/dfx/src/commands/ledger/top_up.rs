use crate::commands::ledger::get_icpts_from_args;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxResult, NotifyTopUpError::Notify};
use crate::lib::ledger_types::Memo;
use crate::lib::ledger_types::NotifyError::Refunded;
use crate::lib::nns_types::account_identifier::Subaccount;
use crate::lib::nns_types::icpts::{ICPTs, TRANSACTION_FEE};
use crate::lib::operations::cmc::{notify_top_up, transfer_cmc};
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::clap::parsers::e8s_parser;
use anyhow::{anyhow, bail, Context};
use candid::Principal;
use clap::Parser;

const MEMO_TOP_UP_CANISTER: u64 = 1347768404_u64;

/// Top up a canister with cycles minted from ICP
#[derive(Parser)]
pub struct TopUpOpts {
    /// Specify the canister id to top up
    canister: String,

    /// Subaccount to withdraw from
    #[arg(long)]
    from_subaccount: Option<Subaccount>,

    /// ICP to mint into cycles and deposit into destination canister
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

    /// Transaction fee, default is 10000 e8s.
    #[arg(long)]
    fee: Option<ICPTs>,

    /// Max fee, default is 10000 e8s.
    #[arg(long)]
    max_fee: Option<ICPTs>,

    /// Transaction timestamp, in nanoseconds, for use in controlling transaction-deduplication, default is system-time. // https://internetcomputer.org/docs/current/developer-docs/integrations/icrc-1/#transaction-deduplication-
    #[arg(long)]
    created_at_time: Option<u64>,
}

pub async fn exec(env: &dyn Environment, opts: TopUpOpts) -> DfxResult {
    let amount = get_icpts_from_args(opts.amount, opts.icp, opts.e8s)?;

    let fee = opts.fee.unwrap_or(TRANSACTION_FEE);

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

    let height = transfer_cmc(
        agent,
        memo,
        amount,
        fee,
        opts.from_subaccount,
        to,
        opts.created_at_time,
    )
    .await?;
    println!("Using transfer at block height {height}");
    let result = notify_top_up(agent, to, height).await;

    match result {
        Ok(cycles) => {
            println!("Canister was topped up with {cycles} cycles!");
        }
        Err(Notify(Refunded {
            reason,
            block_index,
        })) => match block_index {
            Some(height) => {
                println!("Refunded at block height {height} with message: {reason}")
            }
            None => println!("Refunded with message: {reason}"),
        },
        Err(other) => bail!("{other:?}"),
    };
    Ok(())
}
