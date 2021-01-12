use humanize_rs::bytes::{Bytes, Unit};

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

pub fn compute_allocation_validator(compute_allocation: &str) -> Result<(), String> {
    if let Ok(num) = compute_allocation.parse::<u64>() {
        if num <= 100 {
            return Ok(());
        }
    }
    Err("Must be a percent between 0 and 100".to_string())
}

pub fn memory_allocation_validator(memory_allocation: &str) -> Result<(), String> {
    let limit = Bytes::new(256, Unit::TByte).map_err(|_| "Parse Overflow.")?;
    if let Ok(bytes) = memory_allocation.parse::<Bytes>() {
        if bytes.size() <= limit.size() {
            return Ok(());
        }
    }
    Err("Must be a value between 0..256 TB inclusive.".to_string())
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
                    m.iter()
                        .fold(String::new(), |acc, &num| acc + &num.to_string())
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
