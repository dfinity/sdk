use crate::commands::wallet::wallet_update;
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
    wallet_update(env, "name", opts.name.clone()).await?;
    println!("Set name to {}.", opts.name);
    Ok(())
}
