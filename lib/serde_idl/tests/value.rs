extern crate serde_idl;

use serde_idl::{Decode};
use serde_idl::value::IDLValue;

#[test]
fn test_decode() {
    /*
    check(IDLValue::Bool(true), "4449444c00017e01");
    check(IDLValue::Int(1_234_567_890), "4449444c00017cd285d8cc04");
    check(IDLValue::Opt(Box::new(IDLValue::Int(42))), "4449444c016e7c0100012a");
    check(IDLValue::Text("Hi ☃\n".to_string()), "4449444c00017107486920e298830a");
    check(int_vec(&vec![0, 1, 2, 3]), "4449444c016d7c01000400010203");
    check(
        IDLValue::Int(1),
    "4449444c016c02007c017101002a04f09f92a9");*/
    check(IDLValue::Int(1), "4449444c016c02d3e3aa027e868eb7027c0100012a");
}

fn check(v: IDLValue, bytes: &str) {
    let bytes = hex(bytes);
    Decode!(&bytes, decoded: IDLValue);
    assert_eq!(decoded, v);
}

fn int_vec(v: &Vec<i64>) -> IDLValue {
    let vec: Vec<_> = v.iter().map(|n| IDLValue::Int(*n)).collect();
    IDLValue::Vec(vec)
}

fn hex(bytes: &str) -> Vec<u8> {
    hex::decode(bytes).unwrap()
}
