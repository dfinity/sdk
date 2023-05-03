use super::super::notify_create;
use crate::lib::ledger_types::NotifyError;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::{environment::Environment, error::DfxResult};

use anyhow::{anyhow, bail};
use candid::Principal;
use clap::Parser;

#[derive(Parser)]
pub struct NotifyCreateOpts {
    /// BlockHeight at which the send transation was recorded.
    block_height: u64,

    /// The controller of the created canister.
    controller: Principal,

    /// Specify the optional subnet type to create the canister on. If no
    /// subnet type is provided, the canister will be created on a random
    /// default application subnet.
    #[arg(long)]
    subnet_type: Option<String>,
}

pub async fn exec(env: &dyn Environment, opts: NotifyCreateOpts) -> DfxResult {
    // validated by e8s_validator
    let block_height = opts.block_height;
    let controller = opts.controller;

    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    fetch_root_key_if_needed(env).await?;

    let result = notify_create(agent, controller, block_height, opts.subnet_type).await?;

    match result {
        Ok(principal) => {
            println!("Canister created with id: {:?}", principal.to_text());
        }
        Err(NotifyError::Refunded {
            reason,
            block_index,
        }) => {
            match block_index {
                Some(height) => {
                    println!("Refunded at block height {height} with message: {reason}")
                }
                None => println!("Refunded with message: {reason}"),
            };
        }
        Err(other) => bail!("{other:?}"),
    };
    Ok(())
}
