use crate::commands::wallet::wallet_query;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::Clap;
use ic_types::Principal;

/// List the wallet's controllers.
#[derive(Clap)]
pub struct ControllersOpts {}

pub async fn exec(env: &dyn Environment, _opts: ControllersOpts) -> DfxResult {
    let (controllers,): (Vec<Principal>,) = wallet_query(env, "get_controllers", ()).await?;
    for controller in controllers.iter() {
        println!("{}", controller);
    }
    Ok(())
}
