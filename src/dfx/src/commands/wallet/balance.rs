use crate::commands::wallet::get_wallet;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::Parser;
use rust_decimal::prelude::*;

/// Get the cycle balance of the selected Identity's cycles wallet.
#[derive(Parser)]
pub struct WalletBalanceOpts {
    /// Get balance raw value (without upscaling to trillions of cycles).
    #[clap(long)]
    precise: bool,
}

pub async fn exec(env: &dyn Environment, _opts: WalletBalanceOpts) -> DfxResult {
    let balance = get_wallet(env).await?.wallet_balance().await?;
    if _opts.precise {
        println!("{} cycles.", balance.amount);
    } else {
        println!(
            "{} TC (trillion cycles).",
            round_to_trillion_cycles(balance.amount)
        );
    }
    Ok(())
}

fn round_to_trillion_cycles(amount: u128) -> String {
    const SCALE: u32 = 12; // trillion = 10^12
    const FRACTIONAL_PRECISION: u32 = 3;

    // handling edge case when wallet has more than ~99999999999999999999999999999 cycles:
    // ::from_u128() returns None if the value is too big to be handled by rust_decimal,
    // in such case, the integer will be simply divided by 10^(SCALE-FRACTIONAL_PRECISION)
    // and returned as int with manually inserted comma character, therefore sacrificing
    // the fractional precision rounding (which is otherwise provided by rust_decimal)
    let value: String = if let Some(mut dec) = Decimal::from_u128(amount) {
        // safe to .unwrap(), because .set_scale() throws Error only when
        // precision argument is bigger than 28, in our case it's always 12
        dec.set_scale(SCALE).unwrap();
        format!("{}", dec.round_dp(FRACTIONAL_PRECISION))
    } else {
        let mut v = (amount / 10u128.pow(SCALE - FRACTIONAL_PRECISION)).to_string();
        v.insert(v.len() - FRACTIONAL_PRECISION as usize, '.');
        v
    };

    pretty_thousand_separators(value)
}

fn pretty_thousand_separators(i: String) -> String {
    // 1. walk backwards (reverse string) and return characters until comma is seen
    // 2. once comma is seen, start counting chars and:
    //   - every third character but not at the end of the string: return (char + separator)
    //   - otherwise: return char
    // 3. re-reverse the string
    const SEPARATOR: char = ',';
    let mut count: u32 = 0;
    let mut seen_comma = false;
    i.chars()
        .rev()
        .enumerate()
        .map(|(idx, c)| {
            if c == '.' {
                seen_comma = true;
                count += 1;
                c.to_string()
            } else if seen_comma && count.rem_euclid(3) == 0 && count > 0 && i.len() != idx + 1 {
                count += 1;
                format!("{}{}", c, SEPARATOR)
            } else if count == 0 {
                c.to_string()
            } else {
                count += 1;
                c.to_string()
            }
        })
        .collect::<String>()
        .chars()
        .rev()
        .collect::<_>()
}

