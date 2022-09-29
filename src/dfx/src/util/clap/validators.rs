use crate::lib::error::DfxResult;
use anyhow::Context;
use byte_unit::{Byte, ByteUnit};
use ic_utils::interfaces::management_canister::builders::{ComputeAllocation, MemoryAllocation};

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

pub fn trillion_cycle_amount_parser(cycles: &str) -> DfxResult<u128> {
    cycles.parse::<u128>()
        .context("Must be a non negative amount. Currently only accepts whole numbers. Use --cycles otherwise.")
        .and_then(|tc| tc.checked_mul(1_000_000_000_000).context("Too large. Must be less than 10^27"))
}

pub fn compute_allocation_parser(compute_allocation: &str) -> DfxResult<ComputeAllocation> {
    let num = compute_allocation.parse::<u8>()?;
    Ok(ComputeAllocation::try_from(num)?)
}

pub fn memory_allocation_parser(memory_allocation: &str) -> Result<MemoryAllocation, String> {
    // This limit should track MAX_MEMORY_ALLOCATION
    // at https://gitlab.com/dfinity-lab/core/ic/-/blob/master/rs/types/types/src/lib.rs#L492
    let limit = Byte::from_unit(12., ByteUnit::GiB).expect("Parse Overflow.");
    if let Ok(bytes) = memory_allocation.parse::<Byte>() {
        if bytes.get_bytes() <= limit.get_bytes() {
            // MemoryAllocation takes 256TiB, unwrap is safe for <12GiB and bytes are <u64::MAX
            return Ok(MemoryAllocation::try_from(bytes.get_bytes() as u64).unwrap());
        }
    }
    Err("Must be a value between 0..12 GiB inclusive.".to_string())
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
