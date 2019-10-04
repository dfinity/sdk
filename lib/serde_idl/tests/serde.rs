#[macro_use]
extern crate serde_idl;
extern crate dfx_info;

use dfx_info::IDLType;
use serde_idl::Deserialize;

#[test]
fn test_error() {
    check_error(
        || test_decode(b"DID", &42),
        "wrong magic number [68, 73, 68, 0]",
    );
    check_error(
        || test_decode(b"DIDL", &42),
        "leb128::read::Error: failed to fill whole buffer");
    check_error(
        || test_decode(b"DIDL\0\0", &42),
        "No more values to deserialize",
    );
    check_error(
        || test_decode(b"DIDL\x01\x7c", &42),
        "Unsupported op_code -4 in type table",
    );
    // Infinite loop are prevented by design
    check_error(
        || test_decode(b"DIDL\x02\x6e\x01\0", &42),
        "Unsupported op_code 0 in type table",
    );
    check_error(
        || test_decode(b"DIDL\0\x01\x7e\x01\x01", &true),
        "Trailing bytes [1]",
    );
    check_error(
        || test_decode(b"DIDL\0\x01\0\x01", &42),
        "index out of bounds: the len is 0 but the index is 0",
    );
}

#[test]
fn test_bool() {
    all_check(true, "4449444c00017e01");
    all_check(false, "4449444c00017e00");
}

#[test]
fn test_integer() {
    all_check(42, "4449444c00017c2a");
    all_check(1_234_567_890, "4449444c00017cd285d8cc04");
    all_check(-1_234_567_890, "4449444c00017caefaa7b37b");
    all_check(Box::new(42), "4449444c00017c2a");
    check_error(
        || test_decode(&hex::decode("4449444c00017c2a").unwrap(), &42u32),
        "Type mismatch. Type on the wire: Int; Provided type: Nat",
    );
}

#[test]
fn test_text() {
    all_check("Hi ☃\n".to_string(), "4449444c00017107486920e298830a");
    let bytes = hex::decode("4449444c00017107486920e298830a").unwrap();
    test_encode(&"Hi ☃\n", &bytes);
    test_decode(&bytes, &"Hi ☃\n");
}

#[test]
fn test_option() {
    all_check(Some(42), "4449444c016e7c0100012a");
    all_check(Some(Some(42)), "4449444c026e016e7c010001012a");
    let opt: Option<i32> = None;
    all_check(opt, "4449444c016e7c010000");
    // Deserialize \mu T.Option<T> to a non-recursive type
    let v: Option<Option<Option<i32>>> = Some(Some(None));
    test_decode(b"DIDL\x01\x6e\0\x01\0\x01\x01\0", &v);
}

#[test]
fn test_struct() {
    #[derive(PartialEq, Debug, Deserialize, IDLType)]
    struct A1 {
        foo: i32,
        bar: bool,
    }
    let a1 = A1 { foo: 42, bar: true };
    all_check(a1, "4449444c016c02d3e3aa027e868eb7027c0100012a");
    #[derive(PartialEq, Debug, Deserialize, IDLType)]
    struct A11 {
        foo: i32,
        bar: bool,
        baz: A1,
    }
    all_check(
        A11 {
            foo: 42,
            bar: true,
            baz: A1 {
                foo: 10,
                bar: false,
            },
        },
        "4449444c026c03d3e3aa027edbe3aa0201868eb7027c6c02d3e3aa027e868eb7027c010001000a2a",
    );
    
    #[derive(PartialEq, Debug, Deserialize, IDLType)]
    struct A2 {
        foo: i32,
        bar: bool,
        baz: u32,
        bbb: u32,
        bib: u32,
        bab: A1,
    }
    let a1 = A1 { foo: 42, bar: true };
    let a2 = A2 { foo: 42, bar: true, baz: 1, bbb: 1, bib: 1, bab: A1 {foo: 10, bar: false } };
    let bytes = Encode!(&a2);
    test_decode(&bytes, &a1);
    let bytes = Encode!(&a1);
    check_error(|| test_decode(&bytes, &a2), "missing field `baz`");

    #[derive(PartialEq, Debug, Deserialize, IDLType)]
    struct B(bool, i32);
    all_check(B(true, 42), "4449444c016c02007e017c0100012a");

    #[derive(PartialEq, Debug, Deserialize, IDLType)]
    struct List {
        head: i32,
        tail: Option<Box<List>>,
    }

    let list = List {
        head: 42,
        tail: Some(Box::new(List {
            head: 43,
            tail: None,
        })),
    };
    all_check(
        list,
        "4449444c026c02a0d2aca8047c90eddae704016e0001002a012b00",
    );

    let list: Option<List> = None;
    // without memoization on the unrolled type, type table will have 3 entries.
    all_check(list, "4449444c026e016c02a0d2aca8047c90eddae70400010000");
}

#[test]
fn test_mutual_recursion() {
    type List = Option<ListA>;
    #[derive(PartialEq, Debug, Deserialize, IDLType)]
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
    // Space bomb!
    all_check(vec![(); 1000], "4449444c016d7f0100e807");
}

#[test]
fn test_tuple() {
    all_check(
        (42, "💩".to_string()),
        "4449444c016c02007c017101002a04f09f92a9",
    );
    check_error(
        || {
            test_decode(
                &hex::decode("4449444c016c02007c027101002a04f09f92a9").unwrap(),
                &(42, "💩"),
            )
        },
        "Expect vector index 1, but get 2",
    );
}

#[test]
fn test_variant() {
    #[derive(PartialEq, Debug, Deserialize, IDLType)]
    enum Unit {
        Foo,
        Bar,
    }
    all_check(Unit::Bar, "4449444c016b02b3d3c9017fe6fdd5017f010000");
    check_error(
        || {
            test_decode(
                &hex::decode("4449444c016b02b3d3c9017fe6fdd5017f010003").unwrap(),
                &Unit::Bar,
            )
        },
        "variant index 3 larger than length 2",
    );

    #[derive(PartialEq, Debug, Deserialize, IDLType)]
    enum Unit2 {
        Foo,
        Bar,
        Baz,
    }
    let bytes = Encode!(&Unit2::Bar);
    test_decode(&bytes, &Unit::Bar);

    let res: Result<String, String> = Ok("good".to_string());
    all_check(res, "4449444c016b02bc8a0171c5fed2017101000004676f6f64");

    #[allow(dead_code)]
    #[derive(PartialEq, Debug, Deserialize, IDLType)]
    enum E {
        Foo,
        Bar(bool, i32),
        Baz { a: i32, b: u32 },
    }

    let v = E::Bar(true, 42);
    all_check(
        v,
        "4449444c036b03b3d3c90101bbd3c90102e6fdd5017f6c02007e017c6c02617c627d010000012a",
    );
}

#[test]
fn test_generics() {
    #[derive(PartialEq, Debug, Deserialize, IDLType)]
    struct G<T, E> {
        g1: T,
        g2: E,
    }

    let res = G { g1: 42, g2: true };
    all_check(res, "4449444c016c02eab3017cebb3017e01002a01")
}

#[test]
fn test_multiargs() {
    let bytes = Encode!(&42, &Some(42), &Some(1), &Some(2));
    assert_eq!(
        bytes,
        hex::decode("4449444c016e7c047c0000002a012a01010102").unwrap()
    );

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

    check_error(
        || test_decode(&bytes, &42),
        "3 more values need to be deserialized",
    );

    let bytes = Encode!(&[(42, "text")], &(42, "text"));
    assert_eq!(
        bytes,
        hex::decode("4449444c026d016c02007c0171020001012a04746578742a0474657874").unwrap()
    );

    Decode!(&bytes, a: Vec<(i64, &str)>, b: (i64, String));
    assert_eq!(a, [(42, "text")]);
    assert_eq!(b, (42, "text".to_string()));

    let err = || {
        Decode!(&bytes, _a: Vec<(i64, &str)>, _b: (i64, String), _c: i32);
        true
    };
    check_error(err, "No more values to deserialize");
}

fn all_check<T>(value: T, bytes: &str)
where
    T: PartialEq + IDLType + serde::de::DeserializeOwned + std::fmt::Debug,
{
    let bytes = hex::decode(bytes).unwrap();
    test_encode(&value, &bytes);
    test_decode(&bytes, &value);
}

fn test_encode<T>(value: &T, expected: &[u8])
where
    T: IDLType,
{
    let encoded = Encode!(&value);
    assert_eq!(
        encoded, expected,
        "\nActual\n{:x?}\nExpected\n{:x?}\n",
        encoded, expected
    );
}

fn test_decode<'de, T>(bytes: &'de [u8], expected: &T)
where
    T: PartialEq + serde::de::Deserialize<'de> + std::fmt::Debug,
{
    Decode!(bytes, decoded: T);
    assert_eq!(decoded, *expected);
}

fn check_error<F: FnOnce() -> R + std::panic::UnwindSafe, R>(f: F, str: &str) {
    assert_eq!(
        std::panic::catch_unwind(f)
            .err()
            .and_then(|a| a.downcast_ref::<String>().map(|s| { s.contains(str) })),
        Some(true)
    );
}
