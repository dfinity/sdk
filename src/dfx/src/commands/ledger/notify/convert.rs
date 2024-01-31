use crate::lib::error::NotifyMintCyclesError;
use crate::lib::ledger_types::NotifyError::{self};
use crate::lib::ledger_types::NotifyMintCyclesSuccess;
use crate::lib::operations::cmc::notify_mint_cycles;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::{environment::Environment, error::DfxResult};
use crate::util::clap::parsers::icrc_subaccount_parser;
use anyhow::bail;
use clap::Parser;
use icrc_ledger_types::icrc1::account::Subaccount as ICRCSubaccount;
use icrc_ledger_types::icrc1::transfer::Memo as ICRCMemo;

#[derive(Parser)]
pub struct ConvertOpts {
    /// BlockHeight at which the send transaction was recorded.
    block_height: u64,

    /// Subaccount to mint cycles to.
    #[arg(long, value_parser = icrc_subaccount_parser)]
    to_subaccount: Option<ICRCSubaccount>,

    /// Memo used when depositing the minted cycles.
    #[arg(long)]
    deposit_memo: Option<u64>,
}

pub async fn exec(env: &dyn Environment, opts: ConvertOpts) -> DfxResult {
    let block_height = opts.block_height;

    let agent = env.get_agent();

    fetch_root_key_if_needed(env).await?;

    let result = notify_mint_cycles(
        agent,
        opts.deposit_memo.map(ICRCMemo::from),
        opts.to_subaccount,
        block_height,
    )
    .await;

    match result {
        Ok(NotifyMintCyclesSuccess {
            minted, balance, ..
        }) => {
            println!(
                "Canister was topped up with {minted} cycles! New balance is {balance} cycles."
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
