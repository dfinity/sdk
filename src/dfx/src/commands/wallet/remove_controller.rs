use crate::commands::wallet::wallet_update;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use anyhow::Context;
use candid::Principal;
use clap::Parser;

/// Remove a wallet controller.
#[derive(Parser)]
pub struct RemoveControllerOpts {
    /// Principal of the controller to remove.
    controller: String,
}

pub async fn exec(env: &dyn Environment, opts: RemoveControllerOpts) -> DfxResult {
    let controller = Principal::from_text(&opts.controller).with_context(|| {
        format!(
            "Failed to parse {:?} as controller principal.",
            opts.controller
        )
    })?;
    wallet_update(env, "remove_controller", controller).await?;
    println!("Removed {} as a controller.", controller);
    Ok(())
}
