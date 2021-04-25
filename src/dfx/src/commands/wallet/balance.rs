use crate::commands::wallet::do_wallet_call;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::Clap;
use ic_utils::interfaces::wallet::BalanceResult;

/// Get the cycle balance of the selected Identity's cycles wallet.
#[derive(Clap)]
pub struct WalletBalanceOpts {}

pub async fn exec(env: &dyn Environment, _opts: WalletBalanceOpts) -> DfxResult {
    let (balance,): (BalanceResult,) = do_wallet_call(env, "wallet_balance", (), true).await?;
    println!("{} cycles.", balance.amount);
    Ok(())
}
