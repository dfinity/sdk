use crate::commands::wallet::wallet_update;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::Clap;
use ic_types::Principal;

/// Remove a wallet controller.
#[derive(Clap)]
pub struct RemoveControllerOpts {
    /// Principal of the controller to remove.
    controller: String,
}

pub async fn exec(env: &dyn Environment, opts: RemoveControllerOpts) -> DfxResult {
    let controller = Principal::from_text(opts.controller)?;
    wallet_update(env, "remove_controller", controller).await?;
    println!("Removed {} as a controller.", controller);
    Ok(())
}
