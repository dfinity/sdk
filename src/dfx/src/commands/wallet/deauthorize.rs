use crate::commands::wallet::do_wallet_call;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::Clap;
use ic_types::Principal;

/// Deauthorize a wallet custodian.
#[derive(Clap)]
pub struct DeauthorizeOpts {
    /// Principal of the custodian to deauthorize.
    custodian: String,
}

pub async fn exec(env: &dyn Environment, opts: DeauthorizeOpts) -> DfxResult {
    let custodian = Principal::from_text(opts.custodian)?;
    do_wallet_call(env, "deauthorize", custodian, false).await?;
    Ok(())
}
