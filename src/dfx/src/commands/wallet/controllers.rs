use crate::commands::wallet::do_wallet_call;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::Clap;
use ic_types::Principal;

/// List the wallet's controllers.
#[derive(Clap)]
pub struct ControllersOpts {}

pub async fn exec(env: &dyn Environment, _opts: ControllersOpts) -> DfxResult {
    let (controllers,): (Vec<Principal>,) =
        do_wallet_call(env, "get_controllers", (), true).await?;
    let text: Vec<String> = controllers.iter().map(Principal::to_text).collect();
    println!("{:?}", text);
    Ok(())
}
