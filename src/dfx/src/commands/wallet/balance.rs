use crate::commands::wallet::get_wallet;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::Parser;

/// Get the cycle balance of the selected Identity's cycles wallet.
#[derive(Parser)]
pub struct WalletBalanceOpts {}

pub async fn exec(env: &dyn Environment, _opts: WalletBalanceOpts) -> DfxResult {
    let balance = get_wallet(env)
        .await?
        .wallet_balance()
        .await?;
    println!("{} cycles.", balance.amount);
    Ok(())
}
