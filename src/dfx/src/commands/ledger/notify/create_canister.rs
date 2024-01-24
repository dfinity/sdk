use crate::lib::error::NotifyCreateCanisterError::Notify;
use crate::lib::ledger_types::NotifyError::Refunded;
use crate::lib::operations::cmc::notify_create;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::{environment::Environment, error::DfxResult};
use crate::util::clap::subnet_selection_opt::SubnetSelectionOpt;
use anyhow::bail;
use candid::Principal;
use clap::Parser;

#[derive(Parser)]
pub struct NotifyCreateOpts {
    /// BlockHeight at which the send transaction was recorded.
    block_height: u64,

    /// The controller of the created canister.
    controller: Principal,

    #[command(flatten)]
    subnet_selection: SubnetSelectionOpt,
}

pub async fn exec(env: &dyn Environment, opts: NotifyCreateOpts) -> DfxResult {
    // validated by e8s_validator
    let block_height = opts.block_height;
    let controller = opts.controller;

    let agent = env.get_agent();

    fetch_root_key_if_needed(env).await?;

    let subnet_selection = opts.subnet_selection.into_subnet_selection();
    let result = notify_create(agent, controller, block_height, subnet_selection).await;

    match result {
        Ok(principal) => {
            println!("Canister created with id: {:?}", principal.to_text());
        }
        Err(Notify(Refunded {
            reason,
            block_index,
        })) => {
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
