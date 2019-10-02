#[macro_use]
extern crate serde_idl;
extern crate dfx_info;
extern crate serde;

use dfx_info::types::{get_type, Type};
use dfx_info::IDLType;
use serde::Deserialize;
use serde_idl::idl_hash;

#[test]
fn test_bool() {
    all_check(true, "4449444c00017e01");
    all_check(false, "4449444c00017e00");
    assert_eq!(get_type(&true), Type::Bool);
}

#[test]
fn test_integer() {
    all_check(42, "4449444c00017c2a");
    all_check(1_234_567_890, "4449444c00017cd285d8cc04");
    all_check(-1_234_567_890, "4449444c00017caefaa7b37b");
    all_check(Box::new(42), "4449444c00017c2a");
    assert_eq!(get_type(&42), Type::Int);
}

#[test]
fn test_text() {
    all_check("Hi ☃\n".to_string(), "4449444c00017107486920e298830a");
    check("Hi ☃\n", "4449444c00017107486920e298830a");
}

#[test]
fn test_option() {
    all_check(Some(42), "4449444c016e7c0100012a");
    all_check(Some(Some(42)), "4449444c026e016e7c010001012a");
    let opt: Option<i32> = None;
    assert_eq!(get_type(&opt), Type::Opt(Box::new(Type::Int)));
    all_check(opt, "4449444c016e7c010000");
}

#[test]
fn test_struct() {
    #[derive(Debug, Deserialize, IDLType)]
    struct A {
        foo: i32,
        bar: bool,
    }

    let record = A { foo: 42, bar: true };
    assert_eq!(
        get_type(&record),
        Type::Record(vec![field("bar", Type::Bool), field("foo", Type::Int),])
    );
    all_check(record, "4449444c016c02d3e3aa027e868eb7027c0100012a");

    #[derive(Debug, Deserialize, IDLType)]
    struct B(bool, i32);
    all_check(B(true, 42), "4449444c016c02007e017c0100012a");

    #[derive(Debug, Deserialize, IDLType)]
    struct List {
        head: i32,
        tail: Option<Box<List>>,
    }

    let list = List {
        head: 42,
        tail: None,
    };
    assert_eq!(
        get_type(&list),
        Type::Record(vec![
            field("head", Type::Int),
            field(
                "tail",
                Type::Opt(Box::new(Type::Knot(dfx_info::types::TypeId::of::<List>())))
            )
        ])
    );
    all_check(list, "4449444c026c02a0d2aca8047c90eddae704016e0001002a00");

    let list: Option<List> = None;
    // without memoization on the unrolled type, type table will have 3 entries.
    all_check(list, "4449444c026e016c02a0d2aca8047c90eddae70400010000");
}

#[test]
fn test_mutual_recursion() {
    type List = Option<ListA>;
    #[derive(Debug, Deserialize, IDLType)]
    struct ListA {
        head: i32,
        tail: Box<List>,
    };

    let list: List = None;
    all_check(list, "4449444c026e016c02a0d2aca8047c90eddae70400010000");
}

#[test]
fn test_vector() {
    all_check(vec![0, 1, 2, 3], "4449444c016d7c01000400010203");
    all_check([0, 1, 2, 3], "4449444c016d7c01000400010203");
    let boxed_array: Box<[i32]> = Box::new([0, 1, 2, 3]);
    all_check(boxed_array, "4449444c016d7c01000400010203");
    all_check(
        [(42, "text".to_string())],
        "4449444c026d016c02007c01710100012a0474657874",
    );
    all_check([[[[()]]]], "4449444c046d016d026d036d7f010001010101");
}

#[test]
fn test_tuple() {
    all_check(
        (42, "💩".to_string()),
        "4449444c016c02007c017101002a04f09f92a9",
    );
}

#[test]
fn test_variant() {
    #[derive(Debug, Deserialize, IDLType)]
    enum Unit {
        Foo,
        Bar,
    }
    all_check(Unit::Bar, "4449444c016b02b3d3c9017fe6fdd5017f010000");

    let res: Result<String, String> = Ok("good".to_string());
    all_check(res, "4449444c016b02bc8a0171c5fed2017101000004676f6f64");

    #[allow(dead_code)]
    #[derive(Debug, Deserialize, IDLType)]
    enum E {
        Foo,
        Bar(bool, i32),
        Baz { a: i32, b: u32 },
    }

    let v = E::Bar(true, 42);
    assert_eq!(
        get_type(&v),
        Type::Variant(vec![
            field(
                "Bar",
                Type::Record(vec![
                    unnamed_field(0, Type::Bool),
                    unnamed_field(1, Type::Int)
                ])
            ),
            field(
                "Baz",
                Type::Record(vec![field("a", Type::Int), field("b", Type::Nat)])
            ),
            field("Foo", Type::Null),
        ])
    );
    all_check(
        v,
        "4449444c036b03b3d3c90101bbd3c90102e6fdd5017f6c02007e017c6c02617c627d010000012a",
    );
}

#[test]
fn test_generics() {
    #[derive(Debug, Deserialize, IDLType)]
    struct G<T, E> {
        g1: T,
        g2: E,
    }

    let res = G { g1: 42, g2: true };
    assert_eq!(
        get_type(&res),
        Type::Record(vec![field("g1", Type::Int), field("g2", Type::Bool)])
    );
    all_check(res, "4449444c016c02eab3017cebb3017e01002a01")
}

#[test]
fn test_multiargs() {
    checks(
        IDL!(&42, &Some(42), &Some(1), &Some(2)),
        "4449444c016e7c047c0000002a012a01010102",
    );
    let bytes = hex::decode("4449444c016e7c047c0000002a012a01010102").unwrap();
    Decode!(
        &bytes,
        a: i32,
        b: Option<i32>,
        c: Option<i32>,
        d: Option<i32>
    );
    assert_eq!(a, 42);
    assert_eq!(b, Some(42));
    assert_eq!(c, Some(1));
    assert_eq!(d, Some(2));

    checks(
        IDL!(&[(42, "text")], &(42, "text")),
        "4449444c026d016c02007c0171020001012a04746578742a0474657874",
    );
    let bytes = hex::decode("4449444c026d016c02007c0171020001012a04746578742a0474657874").unwrap();
    Decode!(&bytes, a: Vec<(i64, &str)>, b: (i64, &str));
    assert_eq!(a, [(42, "text")]);
    assert_eq!(b, (42, "text"));
}

fn check<T>(value: T, expected: &str)
where
    T: IDLType,
{
    let encoded = IDL!(&value);
    checks(encoded, expected);
}

fn all_check<T>(value: T, expected: &str)
where
    T: IDLType + serde::de::DeserializeOwned,
{
    let expected = hex::decode(expected).unwrap();
    Decode!(&expected, decoded: T);
    let encoded_from_value = IDL!(&value);
    let encoded_from_decoded = IDL!(&decoded);
    assert_eq!(
        encoded_from_value, encoded_from_decoded,
        "\nValue\n{:x?}\nDecoded\n{:x?}\n",
        encoded_from_value, encoded_from_decoded
    );
}

fn checks(encoded: Vec<u8>, expected: &str) {
    let expected = hex::decode(expected).unwrap();
    assert_eq!(
        encoded, expected,
        "\nExpected\n{:x?}\nActual\n{:x?}\n",
        expected, encoded
    );
}

fn field(id: &str, ty: Type) -> dfx_info::types::Field {
    dfx_info::types::Field {
        id: id.to_string(),
        hash: idl_hash(id),
        ty,
    }
}

fn unnamed_field(id: u32, ty: Type) -> dfx_info::types::Field {
    dfx_info::types::Field {
        id: id.to_string(),
        hash: id,
        ty,
    }
}
