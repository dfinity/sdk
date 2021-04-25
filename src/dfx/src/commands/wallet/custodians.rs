use crate::commands::wallet::do_wallet_call;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::Clap;
use ic_types::Principal;

/// List the wallet's custodians.
#[derive(Clap)]
pub struct CustodiansOpts {}

pub async fn exec(env: &dyn Environment, _opts: CustodiansOpts) -> DfxResult {
    let (custodians,): (Vec<Principal>,) = do_wallet_call(env, "get_custodians", (), true).await?;
    let text: Vec<String> = custodians.iter().map(Principal::to_text).collect();
    println!("{:?}", text);
    Ok(())
}
