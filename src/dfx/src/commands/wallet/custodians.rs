use crate::commands::wallet::wallet_query;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use anyhow::Context;
use clap::Parser;
use ic_types::Principal;

/// List the wallet's custodians.
#[derive(Parser)]
pub struct CustodiansOpts {}

pub async fn exec(env: &dyn Environment, _opts: CustodiansOpts) -> DfxResult {
    let (custodians,): (Vec<Principal>,) = wallet_query(env, "get_custodians", ())
        .await
        .context("Failed to fetch wallet custodians.")?;
    for custodian in custodians.iter() {
        println!("{}", custodian);
    }
    Ok(())
}
