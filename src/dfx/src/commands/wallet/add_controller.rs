use crate::commands::wallet::wallet_update;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use anyhow::Context;
use candid::Principal;
use clap::Parser;

/// Add a wallet controller.
#[derive(Parser)]
pub struct AddControllerOpts {
    /// Principal of the controller to add.
    controller: String,
}

pub async fn exec(env: &dyn Environment, opts: AddControllerOpts) -> DfxResult {
    let controller = Principal::from_text(&opts.controller).with_context(|| {
        format!(
            "Failed to parse {:?} as controller principal.",
            opts.controller
        )
    })?;
    wallet_update(env, "add_controller", controller).await?;
    println!("Added {} as a controller.", controller);
    Ok(())
}
