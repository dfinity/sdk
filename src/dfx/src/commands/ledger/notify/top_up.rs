use crate::lib::ledger_types::NotifyError;
use crate::lib::operations::cmc::notify_top_up;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::{environment::Environment, error::DfxResult};
use anyhow::{anyhow, bail};
use candid::Principal;
use clap::Parser;

#[derive(Parser)]
pub struct NotifyTopUpOpts {
    /// BlockHeight at which the send transation was recorded.
    block_height: u64,

    /// The principal of the canister to top up.
    canister: Principal,
}

pub async fn exec(env: &dyn Environment, opts: NotifyTopUpOpts) -> DfxResult {
    // validated by e8s_validator
    let block_height = opts.block_height;
    let canister = opts.canister;

    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    fetch_root_key_if_needed(env).await?;

    let result = notify_top_up(agent, canister, block_height).await?;

    match result {
        Ok(cycles) => {
            println!("Canister {canister} topped up with {cycles} cycles");
        }
        Err(NotifyError::Refunded {
            reason,
            block_index,
        }) => match block_index {
            Some(height) => {
                println!("Refunded at block height {height} with message: {reason}");
            }
            None => println!("Refunded with message: {reason}"),
        },
        Err(other) => bail!("{other:?}"),
    }
    Ok(())
}
