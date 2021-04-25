use crate::commands::wallet::do_wallet_call;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::Clap;

/// Get wallet name.
#[derive(Clap)]
pub struct NameOpts {}

pub async fn exec(env: &dyn Environment, _opts: NameOpts) -> DfxResult {
    let (maybe_name,): (Option<String>,) = do_wallet_call(env, "name", (), true).await?;
    println!("{:?}", maybe_name);
    Ok(())
}
