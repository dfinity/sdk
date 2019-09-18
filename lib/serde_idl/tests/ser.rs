extern crate serde_idl;
extern crate serde;
extern crate dfx_info;

use serde_idl::{to_vec};
use dfx_info::{IDLType};
use dfx_info::types::{Type, get_type};

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
fn test_text() {
    check("Hi ☃\n", "4449444c007107486920e298830a");
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
    #[derive(Debug, IDLType)]
    struct A { foo: i32, bar: bool }
    
    let record = A { foo: 42, bar: true };
    assert_eq!(get_type(&record),
               Type::Record(vec![
                   field("bar", Type::Bool),
                   field("foo", Type::Int),                   
               ])
    );
    check(record, "4449444c016c02d3e3aa027e868eb7027c00012a");    

    #[derive(Debug, IDLType)]
    struct B(bool, i32);
    check(B(true,42), "4449444c016c02007e017c00012a");

    #[derive(Debug, IDLType)]
    struct List { head: i32, tail: Option<Box<List>> }
    
    let list = List { head: 42, tail: None };
    assert_eq!(get_type(&list),
               Type::Record(vec![
                   field("head", Type::Int),
                   field("tail", Type::Opt(Box::new(
                       Type::Knot(dfx_info::types::TypeId::of::<List>()))))])               
    );
    check(list, "4449444c026c02a0d2aca8047c90eddae704016e00002a00");    

    let list: Option<List> = None;
    // without memoization on the unrolled type, type table will have 3 entries.
    check(list, "4449444c026e016c02a0d2aca8047c90eddae704000000");
}

#[test]
fn test_mutual_recursion() {
    type List = Option<ListA>;
    #[derive(Debug, IDLType)]
    struct ListA { head: i32, tail: Box<List> };

    let list: List = None;
    check(list, "4449444c026e016c02a0d2aca8047c90eddae704000000");
}

#[test]
fn test_vector() {
    check(vec![0,1,2,3], "4449444c016d7c000400010203");
    check([0,1,2,3], "4449444c016d7c000400010203");
    let boxed_array: Box<[i32]> = Box::new([0,1,2,3]);
    check(boxed_array, "4449444c016d7c000400010203");
    check([(42, "text")], "4449444c026d016c02007c017100012a0474657874");
    check([[[[()]]]], "4449444c046d016d026d036d7f0001010101");
}

#[test]
fn test_tuple() {
    check((42, "💩"), "4449444c016c02007c0171002a04f09f92a9");
}

#[test]
fn test_variant() {
    #[derive(Debug, IDLType)]
    enum Unit { Foo }
    check(Unit::Foo, "4449444c016b01e6fdd5017f0000");

    let res: Result<&str,&str> = Ok("good");
    check(res, "4449444c016b02bc8a0171c5fed20171000004676f6f64");
    
    #[allow(dead_code)]
    #[derive(Debug, IDLType)]
    enum E { Foo, Bar(bool), Baz{a: i32, b: u32} }
    
    let v = E::Foo;
    assert_eq!(get_type(&v),
               Type::Variant(vec![
                   field("Bar", Type::Record(vec![unnamed_field(0, Type::Bool)])),
                   field("Baz", Type::Record(vec![field("a", Type::Int),
                                                  field("b", Type::Nat)])),
                   field("Foo", Type::Null),                   
                   ])
    );
    check(v, "4449444c036b03b3d3c90101bbd3c90102e6fdd5017f6c01007e6c02617c627d0002");
}

#[test]
fn test_generics() {
    #[derive(Debug, IDLType)]
    struct G<T, E> { g1: T, g2: E }
    
    let res = G { g1: 42, g2: true };
    assert_eq!(get_type(&res),
               Type::Record(vec![
                   field("g1", Type::Int),
                   field("g2", Type::Bool)])
    );
    check(res, "4449444c016c02eab3017cebb3017e002a01")
}

fn check<T>(value: T, expected: &str) where T: IDLType {
    let encoded = to_vec(&value).unwrap();
    let expected = hex::decode(expected).unwrap();
    assert_eq!(encoded, expected, "\nExpected\n{:x?}\nActual\n{:x?}\n", expected, encoded);
}

fn field(id: &str, ty: Type) -> dfx_info::types::Field {
    dfx_info::types::Field { id: id.to_string(), hash:idl_hash(id), ty: ty }
}

fn unnamed_field(id: u32, ty: Type) -> dfx_info::types::Field {
    dfx_info::types::Field { id: id.to_string(), hash:id, ty: ty }
}

fn idl_hash(id: &str) -> u32 {
    let mut s: u32 = 0;
    for c in id.chars() {
        s = s.wrapping_mul(223).wrapping_add(c as u32);
    }
    s
}
