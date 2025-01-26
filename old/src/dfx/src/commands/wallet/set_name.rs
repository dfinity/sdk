use crate::commands::wallet::wallet_update;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use clap::Parser;

/// Set wallet name.
#[derive(Parser)]
pub struct SetNameOpts {
    /// Name of the wallet.
    name: String,
}

pub async fn exec(env: &dyn Environment, opts: SetNameOpts) -> DfxResult {
    wallet_update(env, "set_name", opts.name.clone()).await?;
    println!("Set name to {}.", opts.name);
    Ok(())
}
