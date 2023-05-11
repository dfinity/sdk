use byte_unit::{Byte, ByteUnit};
use std::path::PathBuf;

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

pub fn e8s_parser(e8s: &str) -> Result<u64, String> {
    e8s.parse::<u64>()
        .map_err(|_| "Must specify a non negative whole number.".to_string())
}

pub fn memo_parser(memo: &str) -> Result<u64, String> {
    memo.parse::<u64>()
        .map_err(|_| "Must specify a non negative whole number.".to_string())
}

pub fn cycle_amount_parser(cycles: &str) -> Result<u128, String> {
    cycles
        .parse::<u128>()
        .map_err(|_| "Must be a non negative amount.".to_string())
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

pub fn trillion_cycle_amount_parser(cycles: &str) -> Result<u128, String> {
    format!("{}000000000000", cycles).parse::<u128>()
        .map_err(|_| "Must be a non negative amount. Currently only accepts whole numbers. Use --cycles otherwise.".to_string())
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

pub fn freezing_threshold_parser(freezing_threshold: &str) -> Result<u64, String> {
    freezing_threshold
        .parse::<u64>()
        .map_err(|_| "Must be a value between 0 and 2^64-1 inclusive".to_string())
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
                .matches(|x: char| !x.is_ascii_alphanumeric() && x != '_')
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

pub fn hsm_key_id_parser(key_id: &str) -> Result<String, String> {
    if key_id.len() % 2 != 0 {
        Err("Key id must consist of an even number of hex digits".to_string())
    } else if key_id.contains(|c: char| !c.is_ascii_hexdigit()) {
        Err("Key id must contain only hex digits".to_string())
    } else {
        Ok(key_id.to_string())
    }
}
