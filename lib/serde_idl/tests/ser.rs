extern crate serde_idl;
extern crate serde;
extern crate dfx_info;

use serde::Serialize;
use serde_idl::{to_vec};
use dfx_info::{DfinityInfo, Type, get_type};

#[test]
fn test_bool() {
    check(true, "4449444c007e01");
    check(false, "4449444c007e00");
    assert_eq!(get_type(&true), Type::Bool);
}

#[test]
fn test_integer() {
    check(42, "4449444c007c2a");
    check(1234567890, "4449444c007cd285d8cc04");
    check(-1234567890, "4449444c007caefaa7b37b");
    check(Box::new(42), "4449444c007c2a");
    assert_eq!(get_type(&42), Type::Int);
}

#[test]
fn test_option() {
    check(Some(42), "4449444c016e7c00012a");
    check(Some(Some(42)), "4449444c026e016e7c0001012a");
    let opt: Option<i32> = None;
    assert_eq!(get_type(&opt), Type::Opt(Box::new(Type::Int)));
    check(opt, "4449444c016e7c0000");
}

#[test]
fn test_struct() {
    #[derive(Serialize, Debug, DfinityInfo)]
    struct A { foo: i32, bar: bool }
    
    let record = A { foo: 42, bar: true };
    check(record, "4449444c016c02d3e3aa027e868eb7027c00012a");
    let record = A { foo: 42, bar: true };    
    assert_eq!(get_type(&record),
               Type::Record(vec![
                   field("foo", Type::Int),
                   field("bar", Type::Bool)])
    );
    
    #[derive(Serialize, Debug, DfinityInfo)]
    struct List { head: i32, tail: Option<Box<List>> }
    
    let list = List { head: 42, tail: None };
    assert_eq!(get_type(&list),
               Type::Record(vec![
                   field("head", Type::Int),
                   field("tail", Type::Opt(Box::new(
                       Type::Knot(dfx_info::TypeId::of::<List>()))))])               
    );
    check(list, "4449444c026c02a0d2aca8047c90eddae704016e00002a00");    
    let list: Option<List> = None;
    check(list, "4449444c026c02a0d2aca8047c90eddae704016e000000");
}

#[test]
fn test_variant() {
    #[allow(dead_code)]
    #[derive(Serialize, Debug, DfinityInfo)]
    enum E { Foo, Bar(bool), Baz{a: i32, b: u32} }
    
    let v = E::Foo;
    assert_eq!(get_type(&v),
               Type::Variant(vec![
                   field("Foo", Type::Null),
                   field("Bar", Type::Record(vec![field("0", Type::Bool)])),
                   field("Baz", Type::Record(vec![field("a", Type::Int),
                                                  field("b", Type::Nat)])),
                   ])
    );
    //check(v, "4449444c");
}
/*
#[test]
fn test_generics() {
    #[derive(Serialize, Debug, DfinityInfo)]
    struct G<T, E> { g1: T, g2: E }
    
    let res = G { g1: 42, g2: true };
    assert_eq!(get_type(&res),
               Type::Record(vec![
                   field("g1", Type::Int),
                   field("g2", Type::Bool)])
    );
}
*/
fn check<T>(value: T, expected: &str) where T: Serialize + DfinityInfo {
    let encoded = to_vec(&value).unwrap();
    let expected = hex::decode(expected).unwrap();
    assert_eq!(encoded, expected, "\nExpected\n{:x?}\nActual\n{:x?}\n", expected, encoded);
}

fn field(id: &str, ty: Type) -> dfx_info::Field {
    dfx_info::Field { id: id.to_string(), ty: ty }
}

