extern crate serde_idl;
extern crate serde;
extern crate dfx_info;

use serde::Serialize;
use serde_idl::{to_vec};
use dfx_info::DfinityInfo;

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
    check(Box::new(42), "4449444c007c2a");
}
#[test]
fn test_option() {
    check(Some(42), "4449444c016e7c00012a");
    check(Some(Some(42)), "4449444c026e7c6e000101012a");
    //let opt: Option<i32> = None;
    //check(opt, "4449444c");
}

#[derive(Serialize, Debug, DfinityInfo, Copy, Clone)]
struct A { foo: i32, bar: bool }
#[derive(Serialize, Debug, DfinityInfo)]
struct List { head: i32, tail: Option<Box<List>> }
#[derive(Serialize, Debug, DfinityInfo, Copy, Clone)]
enum E { Foo, Bar(bool) }

#[test]
fn test_struct() {
    let record = A { foo: 42, bar: true };
    check(record, "4449444c016c02d3e3aa027e868eb7027c00012a");
    //check(List { head: 42, tail: None }, "4449444c016c02d3");
}

#[test]
fn test_variant() {
    let e1 = E::Foo;
    let e2 = E::Bar(true);
    assert_eq!(e1.get_type(), dfx_info::Type::Bool);
    assert_eq!(e2.get_type(), dfx_info::Type::Bool);
}

fn check<T: Serialize>(value: T, expected: &str) {
    let encoded = to_vec(&value).unwrap();
    let expected = hex::decode(expected).unwrap();
    assert_eq!(encoded, expected, "\nExpected\n{:x?}\nActual\n{:x?}\n", expected, encoded);
}
