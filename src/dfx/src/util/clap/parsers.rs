use byte_unit::{Byte, ByteUnit};
use candid::Principal;
use ic_utils::interfaces::management_canister::LogVisibility;
use icrc_ledger_types::icrc1::account::Subaccount;
use rust_decimal::Decimal;
use std::{path::PathBuf, str::FromStr};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

/// Removes `_`, interprets `k`, `m`, `b`, `t` suffix (case-insensitive)
fn decimal_with_suffix_parser(input: &str) -> Result<Decimal, String> {
    let input = input.replace('_', "").to_lowercase();
    let (number, suffix) = if input
        .chars()
        .last()
        .map(|char| char.is_alphabetic())
        .unwrap_or(false)
    {
        input.split_at(input.len() - 1)
    } else {
        (input.as_str(), "")
    };
    let multiplier: u64 = match suffix {
        "" => Ok(1),
        "k" => Ok(1_000),
        "m" => Ok(1_000_000),
        "b" => Ok(1_000_000_000),
        "t" => Ok(1_000_000_000_000),
        other => Err(format!("Unknown amount specifier: '{}'", other)),
    }?;
    let number = Decimal::from_str(number).map_err(|err| err.to_string())?;
    Decimal::from(multiplier)
        .checked_mul(number)
        .ok_or_else(|| "Amount too large.".to_string())
}

pub fn request_id_parser(v: &str) -> Result<String, String> {
    // A valid Request Id starts with `0x` and is a series of 64 hexadecimals.
    if !v.starts_with("0x") {
        Err(String::from("A Request ID needs to start with 0x."))
    } else if v.len() != 66 {
        Err(String::from(
            "A Request ID is 64 hexadecimal prefixed with 0x.",
        ))
    } else if v[2..].contains(|c: char| !c.is_ascii_hexdigit()) {
        Err(String::from(
            "A Request ID is 64 hexadecimal prefixed with 0x. An invalid character was found.",
        ))
    } else {
        Ok(v.into())
    }
}

pub fn e8s_parser(input: &str) -> Result<u64, String> {
    decimal_with_suffix_parser(input)?
        .try_into()
        .map_err(|_| "Must specify a non-negative whole number.".to_string())
}

pub fn memo_parser(memo: &str) -> Result<u64, String> {
    memo.parse::<u64>()
        .map_err(|_| "Must specify a non negative whole number.".to_string())
}

pub fn cycle_amount_parser(input: &str) -> Result<u128, String> {
    let removed_cycle_suffix = if input.to_lowercase().ends_with('c') {
        &input[..input.len() - 1]
    } else {
        input
    };

    decimal_with_suffix_parser(removed_cycle_suffix)?.try_into().map_err(|_| "Failed to parse amount. Please use digits only or something like 3.5TC, 2t, or 5_000_000.".to_string())
}

pub fn file_parser(path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);
    if path.exists() {
        Ok(path)
    } else {
        Err("Path does not exist or is not a file.".to_string())
    }
}

pub fn file_or_stdin_parser(path: &str) -> Result<PathBuf, String> {
    if path == "-" {
        // represents stdin
        Ok(PathBuf::from(path))
    } else {
        file_parser(path)
    }
}

pub fn directory_parser(path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);
    if path.is_dir() {
        Ok(path)
    } else {
        Err(format!("Path '{path}' does not exist or is not a directory."))
    }
}

pub fn trillion_cycle_amount_parser(input: &str) -> Result<u128, String> {
    if let Ok(cycles) = format!("{}000000000000", input.replace('_', "")).parse::<u128>() {
        Ok(cycles)
    } else {
        decimal_with_suffix_parser(input)?
            .checked_mul(1_000_000_000_000_u64.into())
            .and_then(|total| total.try_into().ok())
            .ok_or_else(|| "Amount too large.".to_string())
    }
}

pub fn compute_allocation_parser(compute_allocation: &str) -> Result<u64, String> {
    if let Ok(num) = compute_allocation.parse::<u64>() {
        if num <= 100 {
            return Ok(num);
        }
    }
    Err("Must be a percent between 0 and 100".to_string())
}

pub fn memory_allocation_parser(memory_allocation: &str) -> Result<Byte, String> {
    // This limit should track MAX_MEMORY_ALLOCATION
    // at https://gitlab.com/dfinity-lab/core/ic/-/blob/master/rs/types/types/src/lib.rs#L492
    let limit = Byte::from_unit(12., ByteUnit::GiB).expect("Parse Overflow.");
    if let Ok(bytes) = memory_allocation.parse::<Byte>() {
        if bytes <= limit {
            return Ok(bytes);
        }
    }
    Err("Must be a value between 0..12 GiB inclusive.".to_string())
}

pub fn wasm_memory_limit_parser(memory_limit: &str) -> Result<Byte, String> {
    let limit = Byte::from_unit(256., ByteUnit::TiB).expect("Parse Overflow.");
    if let Ok(bytes) = memory_limit.parse::<Byte>() {
        if bytes <= limit {
            return Ok(bytes);
        }
    }
    Err("Must be a value between 0..256 TiB inclusive (e.g. `2GiB`).".to_string())
}

pub fn log_visibility_parser(log_visibility: &str) -> Result<LogVisibility, String> {
    match log_visibility {
        "public" => Ok(LogVisibility::Public),
        "controllers" => Ok(LogVisibility::Controllers),
        _ => Err("Must be `controllers` or `public`.".to_string()),
    }
}

pub fn principal_parser(principal_text: &str) -> Result<Principal, String> {
    match Principal::from_text(principal_text) {
        Ok(principal) => Ok(principal),
        _ => Err(("Failed to convert to a principal.").to_string()),
    }
}

pub fn freezing_threshold_parser(freezing_threshold: &str) -> Result<u64, String> {
    freezing_threshold
        .parse::<u64>()
        .map_err(|_| "Must be a value between 0 and 2^64-1 inclusive".to_string())
}

pub fn reserved_cycles_limit_parser(reserved_cycles_limit: &str) -> Result<u128, String> {
    reserved_cycles_limit
        .parse::<u128>()
        .map_err(|_| "Must be a value between 0 and 2^128-1 inclusive".to_string())
}

/// Validate a String can be a valid project name.
/// A project name is valid if it starts with a letter, and is alphanumeric (with hyphens).
/// It cannot end with a dash.
pub fn project_name_parser(name: &str) -> Result<String, String> {
    let mut chars = name.chars();
    // Check first character first. If there's no first character it's empty.
    if let Some(first) = chars.next() {
        if first.is_ascii_alphabetic() {
            // Then check all other characters.
            // Reverses the search here; if there is a character that is not compatible
            // it is found and an error is returned.
            let m: Vec<&str> = name
                .matches(|x: char| !x.is_ascii_alphanumeric() && x != '_' && x != '-')
                .collect();

            if m.is_empty() {
                Ok(name.to_string())
            } else {
                Err(format!(
                    r#"Invalid character(s): "{}""#,
                    m.iter().fold(String::new(), |acc, &num| acc + num)
                ))
            }
        } else {
            Err("Must start with a letter.".to_owned())
        }
    } else {
        Err("Cannot be empty.".to_owned())
    }
}

pub fn icrc_subaccount_parser(subaccount: &str) -> Result<Subaccount, String> {
    if let Ok(Ok(subaccount)) = hex::decode(subaccount).map(|bytes| bytes.try_into()) {
        return Ok(subaccount);
    }

    Err("Failed to parse subaccount. Expected 32 bytes of hex-encoded data.".to_string())
}

pub fn hsm_key_id_parser(key_id: &str) -> Result<String, String> {
    if key_id.len() % 2 != 0 {
        Err("Key id must consist of an even number of hex digits".to_string())
    } else if key_id.contains(|c: char| !c.is_ascii_hexdigit()) {
        Err("Key id must contain only hex digits".to_string())
    } else {
        Ok(key_id.to_string())
    }
}

/// Parse a duration string into a number of seconds.
/// The duration string is a number followed by a unit (s, m, h, d).
/// The unit is optional, and defaults to seconds.
pub fn duration_parser(duration: &str) -> Result<u64, String> {
    if duration.is_empty() {
        return Err("Duration cannot be empty".to_string());
    }
    let duration = duration.trim().to_lowercase();

    let (number_str, unit) = if duration
        .chars()
        .last()
        .map(|c| c.is_alphabetic())
        .unwrap_or(false)
    {
        duration.split_at(duration.len() - 1)
    } else {
        (duration.as_str(), "s")
    };

    let number = number_str
        .parse::<u64>()
        .map_err(|_| "Invalid number format in duration".to_string())?;

    let seconds = match unit {
        "s" => number,
        "m" => number * 60,
        "h" => number * 3600,
        "d" => number * 86400,
        _ => {
            return Err(format!(
                "Unknown duration unit: '{}'. Valid units are s, m, h, d",
                unit
            ))
        }
    };

    Ok(seconds)
}

/// Parse a timestamp string that can be either RFC3339 format or nanoseconds since epoch.
/// Returns the timestamp in nanoseconds since epoch.
pub fn timestamp_parser(timestamp: &str) -> Result<u64, String> {
    if let Ok(nanos) = timestamp.parse::<u64>() {
        return Ok(nanos);
    }

    if let Ok(time) = OffsetDateTime::parse(timestamp, &Rfc3339) {
        let nanos = time.unix_timestamp_nanos();
        let nanos = nanos
            .try_into()
            .map_err(|_| format!("Timestamp {} out of range for u64", nanos))?;
        return Ok(nanos);
    }

    Err(format!("Invalid timestamp format: {}. Required either nanoseconds since epoch or RFC3339 format (e.g. '2021-05-06T19:17:10.000000002Z')", timestamp))
}

#[test]
fn test_cycle_amount_parser() {
    assert_eq!(cycle_amount_parser("900c"), Ok(900));
    assert_eq!(cycle_amount_parser("9_887K"), Ok(9_887_000));
    assert_eq!(cycle_amount_parser("0.1M"), Ok(100_000));
    assert_eq!(cycle_amount_parser("0.01b"), Ok(10_000_000));
    assert_eq!(cycle_amount_parser("10T"), Ok(10_000_000_000_000));
    assert_eq!(cycle_amount_parser("10TC"), Ok(10_000_000_000_000));
    assert_eq!(cycle_amount_parser("1.23t"), Ok(1_230_000_000_000));

    assert!(cycle_amount_parser("1ffff").is_err());
    assert!(cycle_amount_parser("1MT").is_err());
    assert!(cycle_amount_parser("-0.1m").is_err());
    assert!(cycle_amount_parser("T100").is_err());
    assert!(cycle_amount_parser("1.1k0").is_err());
    assert!(cycle_amount_parser(&format!("{}0", u128::MAX)).is_err());
}

#[test]
fn test_trillion_cycle_amount_parser() {
    const TRILLION: u128 = 1_000_000_000_000;
    assert_eq!(trillion_cycle_amount_parser("3"), Ok(3 * TRILLION));
    assert_eq!(trillion_cycle_amount_parser("5_555"), Ok(5_555 * TRILLION));
    assert_eq!(trillion_cycle_amount_parser("1k"), Ok(1_000 * TRILLION));
    assert_eq!(trillion_cycle_amount_parser("0.3"), Ok(300_000_000_000));
    assert_eq!(trillion_cycle_amount_parser("0.3k"), Ok(300 * TRILLION));

    assert!(trillion_cycle_amount_parser("-0.1m").is_err());
    assert!(trillion_cycle_amount_parser("1TC").is_err()); // ambiguous in combination with --t
}

#[test]
fn test_e8s_parser() {
    assert_eq!(e8s_parser("1"), Ok(1));
    assert_eq!(e8s_parser("1_000"), Ok(1_000));
    assert_eq!(e8s_parser("1k"), Ok(1_000));
    assert_eq!(e8s_parser("1M"), Ok(1_000_000));
}

#[test]
fn test_duration_parser() {
    assert_eq!(duration_parser("5s"), Ok(5));
    assert_eq!(duration_parser("5"), Ok(5));
    assert_eq!(duration_parser("2m"), Ok(120));
    assert_eq!(duration_parser("1h"), Ok(3600));
    assert_eq!(duration_parser("1d"), Ok(86400));

    assert!(duration_parser("").is_err());
    assert!(duration_parser("-1s").is_err());
    assert!(duration_parser("abc").is_err());
    assert!(duration_parser("1y").is_err());
}

#[test]
fn test_timestamp_parser() {
    assert_eq!(
        timestamp_parser("1620328630000000002"),
        Ok(1620328630000000002)
    );

    assert_eq!(
        timestamp_parser("2021-05-06T19:17:10.000000002Z"),
        Ok(1620328630000000002)
    );

    assert!(timestamp_parser("invalid format").is_err());
    assert!(timestamp_parser("2021-05-06T19:17:10.000000002").is_err()); // Missing timezone
    assert!(timestamp_parser("2021-13-01T00:00:00Z").is_err()); // Invalid month
}
