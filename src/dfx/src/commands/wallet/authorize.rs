use crate::commands::wallet::do_wallet_call;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::Clap;
use ic_types::Principal;

/// Authorize a wallet custodian.
#[derive(Clap)]
pub struct AuthorizeOpts {
    /// Principal of the custodian to authorize.
    custodian: String,
}

pub async fn exec(env: &dyn Environment, opts: AuthorizeOpts) -> DfxResult {
    let custodian = Principal::from_text(opts.custodian.clone())?;
    do_wallet_call(env, "authorize", custodian, false).await?;
    println!("Authorized {} as a custodian.", opts.custodian);
    Ok(())
}
