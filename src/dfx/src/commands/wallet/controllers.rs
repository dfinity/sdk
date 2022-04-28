use crate::commands::wallet::wallet_query;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use anyhow::Context;
use clap::Parser;
use ic_types::Principal;

/// List the wallet's controllers.
#[derive(Parser)]
pub struct ControllersOpts {}

pub async fn exec(env: &dyn Environment, _opts: ControllersOpts) -> DfxResult {
    let (controllers,): (Vec<Principal>,) = wallet_query(env, "get_controllers", ())
        .await
        .context("Failed to fetch wallet controllers.")?;
    for controller in controllers.iter() {
        println!("{}", controller);
    }
    Ok(())
}
