extern crate serde_idl;

use serde_idl::value::{IDLField, IDLValue};
use serde_idl::Decode;

#[test]
fn test() {
    use IDLValue::*;
    check(Bool(true), "4449444c00017e01");
    check(Int(1_234_567_890), "4449444c00017cd285d8cc04");
    check(Opt(Box::new(Int(42))), "4449444c016e7c0100012a");
    check(Text("Hi ☃\n".to_string()), "4449444c00017107486920e298830a");
    check(int_vec(&[0, 1, 2, 3]), "4449444c016d7c01000400010203");
    check(
        Record(vec![
            IDLField {
                id: 0,
                val: Int(42),
            },
            IDLField {
                id: 1,
                val: Text("💩".to_string()),
            },
        ]),
        "4449444c016c02007c017101002a04f09f92a9",
    );
    check(
        Record(vec![
            IDLField {
                id: 4_895_187,
                val: Bool(true),
            },
            IDLField {
                id: 5_097_222,
                val: Int(42),
            },
        ]),
        "4449444c016c02d3e3aa027e868eb7027c0100012a",
    );
}

#[test]
fn test_variant() {
    use IDLValue::*;
    let value = Variant(Box::new(IDLField {
        id: 3_303_859,
        val: Null,
    }));
    let bytes = hex("4449444c016b02b3d3c9017fe6fdd5017f010000");
    test_decode(&bytes, &value);
    let encoded = serde_idl::encode_value(&[value.clone()]).unwrap();
    test_decode(&encoded, &value);
}

fn check(v: IDLValue, bytes: &str) {
    let bytes = hex(bytes);
    test_decode(&bytes, &v);
    test_encode(&v, &bytes);
}

fn test_encode(v: &IDLValue, expected: &[u8]) {
    let encoded = serde_idl::encode_value(&[v.clone()]).unwrap();
    assert_eq!(
        encoded, expected,
        "\nActual\n{:x?}\nExpected\n{:x?}\n",
        encoded, expected
    );
}

fn test_decode(bytes: &[u8], expected: &IDLValue) {
    Decode!(bytes, decoded: IDLValue);
    assert_eq!(decoded, *expected);
}

fn int_vec(v: &[i64]) -> IDLValue {
    let vec: Vec<_> = v.iter().map(|n| IDLValue::Int(*n)).collect();
    IDLValue::Vec(vec)
}

fn hex(bytes: &str) -> Vec<u8> {
    hex::decode(bytes).unwrap()
}
