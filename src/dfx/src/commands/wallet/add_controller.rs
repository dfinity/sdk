use crate::commands::wallet::wallet_update;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::Clap;
use ic_types::Principal;

/// Add a wallet controller.
#[derive(Clap)]
pub struct AddControllerOpts {
    /// Principal of the controller to add.
    controller: String,
}

pub async fn exec(env: &dyn Environment, opts: AddControllerOpts) -> DfxResult {
    let controller = Principal::from_text(opts.controller)?;
    wallet_update(env, "add_controller", controller).await?;
    println!("Added {} as a controller.", controller);
    Ok(())
}
