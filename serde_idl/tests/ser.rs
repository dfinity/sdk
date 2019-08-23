extern crate serde_idl;
extern crate serde;

use serde::Serialize;
use serde_idl::{to_vec};

#[test]
fn test_bool() {
    check(true, "4449444c007e01");
    check(false, "4449444c007e00");
}

#[test]
fn test_integer() {
    check(42, "4449444c007c2a");
    check(1234567890, "4449444c007cd285d8cc04");
    check(-1234567890, "4449444c007caefaa7b37b");
}
#[test]
fn test_option() {
    check(Some(42), "4449444c016e7c00012a");
    check(Some(Some(42)), "4449444c026e7c6e000101012a");
}

fn check<T: Serialize>(value: T, expected: &str) {
    let encoded = to_vec(&value).unwrap();
    let expected = hex::decode(expected).unwrap();
    assert_eq!(encoded, expected);
}
