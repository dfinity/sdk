extern crate dfx_info;

use dfx_info::types::{get_type, Type};
use dfx_info::{idl_hash, IDLType};

#[test]
fn test_primitive() {
    assert_eq!(get_type(&true), Type::Bool);
    assert_eq!(get_type(&Box::new(42)), Type::Int);
    let opt: Option<&str> = None;
    assert_eq!(get_type(&opt), Type::Opt(Box::new(Type::Text)));
    assert_eq!(get_type(&[0, 1, 2, 3]), Type::Vec(Box::new(Type::Int)));
}

#[test]
fn test_struct() {
    #[derive(Debug, IDLType)]
    struct A {
        foo: i32,
        bar: bool,
    }
    assert_eq!(
        A::ty(),
        Type::Record(vec![field("bar", Type::Bool), field("foo", Type::Int),])
    );

    #[derive(Debug, IDLType)]
    struct G<T, E> {
        g1: T,
        g2: E,
    }
    let res = G { g1: 42, g2: true };
    assert_eq!(
        get_type(&res),
        Type::Record(vec![field("g1", Type::Int), field("g2", Type::Bool)])
    );

    #[derive(Debug, IDLType)]
    struct List {
        head: i32,
        tail: Option<Box<List>>,
    }
    assert_eq!(
        List::ty(),
        Type::Record(vec![
            field("head", Type::Int),
            field(
                "tail",
                Type::Opt(Box::new(Type::Knot(dfx_info::types::TypeId::of::<List>())))
            )
        ])
    );
}

#[test]
fn test_variant() {
    #[allow(dead_code)]
    #[derive(Debug, IDLType)]
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
