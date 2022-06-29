use crate::commands::ledger::notify_top_up;
use crate::lib::ledger_types::NotifyError;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::{environment::Environment, error::DfxResult};
use crate::util::clap::validators::e8s_validator;

use anyhow::{anyhow, bail, Context};
use candid::Principal;
use clap::Parser;

#[derive(Parser)]
pub struct NotifyTopUpOpts {
    /// BlockHeight at which the send transation was recorded.
    #[clap(validator(e8s_validator))]
    block_height: String,

    /// The principal of the canister to top up.
    canister: String,
}

pub async fn exec(env: &dyn Environment, opts: NotifyTopUpOpts) -> DfxResult {
    // validated by e8s_validator
    let block_height = opts.block_height.parse::<u64>().unwrap();
    let canister = Principal::from_text(&opts.canister).with_context(|| {
        format!(
            "Failed to parse {:?} as destination principal.",
            opts.canister
        )
    })?;

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
