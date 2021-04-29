use crate::commands::wallet::wallet_query;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::Clap;
use ic_utils::interfaces::wallet::BalanceResult;

/// Get the cycle balance of the selected Identity's cycles wallet.
#[derive(Clap)]
pub struct WalletBalanceOpts {}

pub async fn exec(env: &dyn Environment, _opts: WalletBalanceOpts) -> DfxResult {
    let (balance,): (BalanceResult,) = wallet_query(env, "wallet_balance", ()).await?;
    println!("{} cycles.", balance.amount);
    Ok(())
}
