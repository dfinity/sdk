use crate::commands::wallet::wallet_query;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::Clap;
use ic_types::Principal;

/// List the wallet's custodians.
#[derive(Clap)]
pub struct CustodiansOpts {}

pub async fn exec(env: &dyn Environment, _opts: CustodiansOpts) -> DfxResult {
    let (custodians,): (Vec<Principal>,) = wallet_query(env, "get_custodians", ()).await?;
    for custodian in custodians.iter() {
        println!("{}", custodian);
    }
    Ok(())
}
