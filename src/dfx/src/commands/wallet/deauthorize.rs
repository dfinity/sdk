use crate::commands::wallet::wallet_update;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use anyhow::Context;
use candid::Principal;
use clap::Parser;

/// Deauthorize a wallet custodian.
#[derive(Parser)]
pub struct DeauthorizeOpts {
    /// Principal of the custodian to deauthorize.
    custodian: String,
}

pub async fn exec(env: &dyn Environment, opts: DeauthorizeOpts) -> DfxResult {
    let custodian = Principal::from_text(&opts.custodian).with_context(|| {
        format!(
            "Failed to parse {:?} as custodian principal.",
            opts.custodian
        )
    })?;
    wallet_update(env, "deauthorize", custodian).await?;
    println!("Deauthorized {} as a custodian.", opts.custodian);
    Ok(())
}
