use crate::commands::wallet::do_wallet_call;
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
    let controller = Principal::from_text(opts.controller.clone())?;
    do_wallet_call(env, "add_controller", controller, false).await?;
    println!("Added {} as a controller.", opts.controller);
    Ok(())
}
