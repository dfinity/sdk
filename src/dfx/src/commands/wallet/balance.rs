use crate::commands::wallet::get_wallet;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::Parser;
use num_traits::FromPrimitive;
use rust_decimal::Decimal;

const DECIMAL_POINT: char = '.';

/// Get the cycle balance of the selected Identity's cycles wallet.
#[derive(Parser)]
pub struct WalletBalanceOpts {
    /// Get balance raw value (without upscaling to trillions of cycles).
    #[clap(long)]
    precise: bool,
}

pub async fn exec(env: &dyn Environment, opts: WalletBalanceOpts) -> DfxResult {
    let balance = get_wallet(env).await?.wallet_balance().await?;
    if opts.precise {
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
        v.insert(v.len() - FRACTIONAL_PRECISION as usize, DECIMAL_POINT);
        v
    };

    pretty_thousand_separators(value)
}

fn pretty_thousand_separators(num: String) -> String {
    // 1. walk backwards (reverse string) and return characters until decimal point is seen
    // 2. once decimal point is seen, start counting chars and:
    //   - every third character but not at the end of the string: return (char + delimiter)
    //   - otherwise: return char
    // 3. re-reverse the string
    const GROUP_DELIMITER: char = ',';
    let mut count: u32 = 0;
    let mut seen_decimal_point = false;
    num.chars()
        .rev()
        .enumerate()
        .map(|(idx, c)| {
            if c == DECIMAL_POINT {
                seen_decimal_point = true;
                count += 1;
                c.to_string()
            } else if seen_decimal_point
                && count.rem_euclid(3) == 0
                && count > 0
                && num.len() != idx + 1
            {
                count += 1;
                format!("{}{}", c, GROUP_DELIMITER)
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

#[cfg(test)]
mod tests {
    use super::{pretty_thousand_separators, round_to_trillion_cycles};

    #[test]
    fn prettify_balance_amount() {
        // thousands separator
        assert_eq!(
            "13,333.456",
            pretty_thousand_separators("13333.456".to_string())
        );
        assert_eq!(
            "3,333.456",
            pretty_thousand_separators("3333.456".to_string())
        );
        assert_eq!("333.456", pretty_thousand_separators("333.456".to_string()));
        assert_eq!("33.456", pretty_thousand_separators("33.456".to_string()));
        assert_eq!("3.456", pretty_thousand_separators("3.456".to_string()));

        // rounding + thousands separator
        assert_eq!("1,234.568", round_to_trillion_cycles(1234567890100000));
        assert_eq!(
            "123,456,123,412.348",
            round_to_trillion_cycles(123456123412347890100000)
        );
        assert_eq!("0.000", round_to_trillion_cycles(1234));
        assert_eq!("0.000", round_to_trillion_cycles(0));
        assert_eq!("0.000", round_to_trillion_cycles(500000000));
        assert_eq!("0.001", round_to_trillion_cycles(500000001));
        assert_eq!("12.568", round_to_trillion_cycles(12567890100000));
        assert_eq!(
            "340,282,366,920,938,463,463,374,607.431",
            round_to_trillion_cycles(u128::MAX)
        );
        assert_eq!(
            "99,999,999,999,999,999.999",
            round_to_trillion_cycles(99999999999999999999999999999)
        );
        assert_eq!(
            "10,000,000,000,000,000.000",
            round_to_trillion_cycles(9999999999999999999999999999)
        );
        assert_eq!("1.268", round_to_trillion_cycles(1267890100000));
        assert_eq!("0.168", round_to_trillion_cycles(167890100000));
    }
}
