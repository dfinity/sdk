use crate::commands::wallet::do_wallet_call;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::Clap;

/// Set wallet name.
#[derive(Clap)]
pub struct SetNameOpts {
    /// Name of the wallet.
    name: String,
}

pub async fn exec(env: &dyn Environment, opts: SetNameOpts) -> DfxResult {
    do_wallet_call(env, "name", opts.name.clone(), false).await?;
    println!("Set name to {}.", opts.name);
    Ok(())
}
