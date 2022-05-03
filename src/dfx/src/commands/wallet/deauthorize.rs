use crate::commands::wallet::wallet_update;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use anyhow::Context;
use clap::Parser;
use ic_types::Principal;

/// Deauthorize a wallet custodian.
#[derive(Parser)]
pub struct DeauthorizeOpts {
    /// Principal of the custodian to deauthorize.
    custodian: String,
}

pub async fn exec(env: &dyn Environment, opts: DeauthorizeOpts) -> DfxResult {
    let custodian =
        Principal::from_text(&opts.custodian).context("Failed to parse custodian principal.")?;
    wallet_update(env, "deauthorize", custodian).await?;
    println!("Deauthorized {} as a custodian.", opts.custodian);
    Ok(())
}
