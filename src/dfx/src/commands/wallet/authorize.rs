use crate::commands::wallet::wallet_update;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use anyhow::Context;
use clap::Parser;
use ic_types::Principal;

/// Authorize a wallet custodian.
#[derive(Parser)]
pub struct AuthorizeOpts {
    /// Principal of the custodian to authorize.
    custodian: String,
}

pub async fn exec(env: &dyn Environment, opts: AuthorizeOpts) -> DfxResult {
    let custodian =
        Principal::from_text(opts.custodian).context("Failed to parse custodian principal.")?;
    wallet_update(env, "authorize", custodian)
        .await
        .context("Failed to add custodian to the wallet.")?;
    println!("Authorized {} as a custodian.", custodian);
    Ok(())
}
