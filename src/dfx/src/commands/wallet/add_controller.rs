use crate::commands::wallet::wallet_update;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use anyhow::Context;
use clap::Parser;
use ic_types::Principal;

/// Add a wallet controller.
#[derive(Parser)]
pub struct AddControllerOpts {
    /// Principal of the controller to add.
    controller: String,
}

pub async fn exec(env: &dyn Environment, opts: AddControllerOpts) -> DfxResult {
    let controller =
        Principal::from_text(opts.controller).context("Failed to parse controller principal.")?;
    wallet_update(env, "add_controller", controller)
        .await
        .context("Failed to add controller to the wallet.")?;
    println!("Added {} as a controller.", controller);
    Ok(())
}
