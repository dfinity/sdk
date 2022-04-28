use crate::commands::wallet::wallet_update;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use anyhow::Context;
use clap::Parser;

/// Set wallet name.
#[derive(Parser)]
pub struct SetNameOpts {
    /// Name of the wallet.
    name: String,
}

pub async fn exec(env: &dyn Environment, opts: SetNameOpts) -> DfxResult {
    wallet_update(env, "name", opts.name.clone())
        .await
        .context("Failed to update wallet name.")?;
    println!("Set name to {}.", opts.name);
    Ok(())
}
