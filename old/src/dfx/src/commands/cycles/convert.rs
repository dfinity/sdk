use crate::commands::ledger::get_icpts_from_args;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxResult, NotifyMintCyclesError};
use crate::lib::ledger_types::{Memo as ICPMemo, NotifyError, NotifyMintCyclesSuccess};
use crate::lib::nns_types::account_identifier::Subaccount as ICPSubaccount;
use crate::lib::nns_types::icpts::{ICPTs, TRANSACTION_FEE};
use crate::lib::operations::cmc::{notify_mint_cycles, transfer_cmc};
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::clap::parsers::{e8s_parser, icrc_subaccount_parser};
use anyhow::{anyhow, bail};
use clap::Parser;
use icrc_ledger_types::icrc1::account::Subaccount as ICRCSubaccount;
use icrc_ledger_types::icrc1::transfer::Memo as ICRCMemo;

pub const MEMO_MINT_CYCLES: u64 = 0x544e494d; // == 'MINT'

/// Convert some of the user's ICP balance into cycles.
#[derive(Parser)]
pub struct ConvertOpts {
    /// Subaccount to withdraw from
    #[arg(long)]
    from_subaccount: Option<ICPSubaccount>,

    /// Subaccount to mint cycles to.
    #[arg(long, value_parser = icrc_subaccount_parser)]
    to_subaccount: Option<ICRCSubaccount>,

    /// Transaction fee, default is 10000 e8s.
    #[arg(long)]
    fee: Option<ICPTs>,

    /// Transaction timestamp, in nanoseconds, for use in controlling transaction-deduplication, default is system-time.
    // https://internetcomputer.org/docs/current/developer-docs/integrations/icrc-1/#transaction-deduplication-
    #[arg(long)]
    created_at_time: Option<u64>,

    /// ICP to mint into cycles and deposit into your cycles ledger account
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

    /// Memo used when depositing the minted cycles.
    #[arg(long)]
    deposit_memo: Option<u64>,
}

pub async fn exec(env: &dyn Environment, opts: ConvertOpts) -> DfxResult {
    let amount = get_icpts_from_args(opts.amount, opts.icp, opts.e8s)?;
    let memo = ICPMemo(MEMO_MINT_CYCLES);
    let fee = opts.fee.unwrap_or(TRANSACTION_FEE);
    let agent = env.get_agent();
    let to = agent
        .get_principal()
        .map_err(|err| anyhow!("Failed to get selected identity principal: {err}"))?;

    fetch_root_key_if_needed(env).await?;

    let height = transfer_cmc(
        agent,
        env.get_logger(),
        memo,
        amount,
        fee,
        opts.from_subaccount,
        to,
        opts.created_at_time,
    )
    .await?;
    println!("Using transfer at block height {height}");
    let result = notify_mint_cycles(
        agent,
        opts.deposit_memo.map(ICRCMemo::from),
        opts.to_subaccount,
        height,
    )
    .await;

    match result {
        Ok(NotifyMintCyclesSuccess {
            minted, balance, ..
        }) => {
            println!(
                "Account was topped up with {minted} cycles! New balance is {balance} cycles."
            );
        }
        Err(NotifyMintCyclesError::Notify(NotifyError::Refunded {
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
