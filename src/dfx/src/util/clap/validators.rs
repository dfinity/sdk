use crate::lib::nns_types::icpts::ICPTs;
use byte_unit::{Byte, ByteUnit};
use std::path::Path;
use std::str::FromStr;

pub fn is_request_id(v: &str) -> Result<(), String> {
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
        Ok(())
    }
}

pub fn e8s_validator(e8s: &str) -> Result<(), String> {
    if e8s.parse::<u64>().is_ok() {
        return Ok(());
    }
    Err("Must specify a non negative whole number.".to_string())
}

pub fn icpts_amount_validator(icpts: &str) -> Result<(), String> {
    ICPTs::from_str(icpts).map(|_| ())
}

pub fn memo_validator(memo: &str) -> Result<(), String> {
    if memo.parse::<u64>().is_ok() {
        return Ok(());
    }
    Err("Must specify a non negative whole number.".to_string())
}

pub fn cycle_amount_validator(cycles: &str) -> Result<(), String> {
    if cycles.parse::<u128>().is_ok() {
        return Ok(());
    }
    Err("Must be a non negative amount.".to_string())
}

pub fn file_validator(path: &str) -> Result<(), String> {
    if Path::new(path).exists() {
        return Ok(());
    }
    Err("Path does not exist or is not a file.".to_string())
}

pub fn file_or_stdin_validator(path: &str) -> Result<(), String> {
    if path == "-" {
        // represents stdin
        Ok(())
    } else {
        file_validator(path)
    }
}

pub fn trillion_cycle_amount_validator(cycles: &str) -> Result<(), String> {
    if format!("{}000000000000", cycles).parse::<u128>().is_ok() {
        return Ok(());
    }
    Err("Must be a non negative amount. Currently only accepts whole numbers. Use --cycles otherwise.".to_string())
}

pub fn compute_allocation_validator(compute_allocation: &str) -> Result<(), String> {
    if let Ok(num) = compute_allocation.parse::<u64>() {
        if num <= 100 {
            return Ok(());
        }
    }
    Err("Must be a percent between 0 and 100".to_string())
}

pub fn memory_allocation_validator(memory_allocation: &str) -> Result<(), String> {
    // This limit should track MAX_MEMORY_ALLOCATION
    // at https://gitlab.com/dfinity-lab/core/ic/-/blob/master/rs/types/types/src/lib.rs#L492
    let limit = Byte::from_unit(12., ByteUnit::GiB).expect("Parse Overflow.");
    if let Ok(bytes) = memory_allocation.parse::<Byte>() {
        if bytes.get_bytes() <= limit.get_bytes() {
            return Ok(());
        }
    }
    Err("Must be a value between 0..12 GiB inclusive.".to_string())
}

pub fn freezing_threshold_validator(freezing_threshold: &str) -> Result<(), String> {
    if let Ok(num) = freezing_threshold.parse::<u128>() {
        if num <= (2_u128.pow(64) - 1) {
            return Ok(());
        }
    }
    Err("Must be a value between 0 and 2^64-1 inclusive".to_string())
}

/// Validate a String can be a valid project name.
/// A project name is valid if it starts with a letter, and is alphanumeric (with hyphens).
/// It cannot end with a dash.
pub fn project_name_validator(name: &str) -> Result<(), String> {
    let mut chars = name.chars();
    // Check first character first. If there's no first character it's empty.
    if let Some(first) = chars.next() {
        if first.is_ascii_alphabetic() {
            // Then check all other characters.
            // Reverses the search here; if there is a character that is not compatible
            // it is found and an error is returned.
            let m: Vec<&str> = name
                .matches(|x: char| !x.is_ascii_alphanumeric() && x != '_')
                .collect();

            if m.is_empty() {
                Ok(())
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

pub fn is_hsm_key_id(key_id: &str) -> Result<(), String> {
    if key_id.len() % 2 != 0 {
        Err("Key id must consist of an even number of hex digits".to_string())
    } else if key_id.contains(|c: char| !c.is_ascii_hexdigit()) {
        Err("Key id must contain only hex digits".to_string())
    } else {
        Ok(())
    }
}
