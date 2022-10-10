use crate::commands::wallet::get_wallet;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::util::{format_as_trillions, pretty_thousand_separators};

use anyhow::Context;
use clap::Parser;

/// Get the cycle balance of the selected Identity's cycles wallet.
#[derive(Parser)]
pub struct WalletBalanceOpts {
    /// Get balance raw value (without upscaling to trillions of cycles).
    #[clap(long)]
    precise: bool,
}

pub async fn exec(env: &dyn Environment, opts: WalletBalanceOpts) -> DfxResult {
    let balance = get_wallet(env)
        .await?
        .wallet_balance()
        .await
        .context("Failed to fetch wallet balance.")?;

    if opts.precise {
        println!("{} cycles.", balance.amount);
    } else {
        println!(
            "{} TC (trillion cycles).",
            pretty_thousand_separators(format_as_trillions(balance.amount))
        );
    }

    Ok(())
}
