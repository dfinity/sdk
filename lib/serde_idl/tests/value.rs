extern crate serde_idl;

use serde_idl::{Decode};
use serde_idl::value::{IDLValue, IDLField};

#[test]
fn test_decode() {
    use IDLValue::*;
    check(Bool(true), "4449444c00017e01");
    check(Int(1_234_567_890), "4449444c00017cd285d8cc04");
    check(Opt(Box::new(Int(42))), "4449444c016e7c0100012a");
    check(Text("Hi ☃\n".to_string()), "4449444c00017107486920e298830a");
    check(int_vec(&vec![0, 1, 2, 3]), "4449444c016d7c01000400010203");
    check(Record(vec![IDLField { id: 0, val: Int(42) }, IDLField { id: 1, val: Text("💩".to_string()) }]),
    "4449444c016c02007c017101002a04f09f92a9");
    check(Record(vec![IDLField { id: 4895187, val: Bool(true) }, IDLField { id: 5097222, val: Int(42) }]),
          "4449444c016c02d3e3aa027e868eb7027c0100012a");
    check(Variant(Box::new(IDLField { id: 3303859, val: Null })),
          "4449444c016b02b3d3c9017fe6fdd5017f010000");
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
