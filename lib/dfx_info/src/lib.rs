extern crate dfx_derive;

pub use dfx_derive::*;

#[derive(Debug, PartialEq)]
pub enum Type {
    Null,
    Bool,
    Nat,
    Int,
    Opt(Box<Type>),
    Vec(Box<Type>),
    Record(Vec<Field>),
    Variant(Vec<Field>),
}

#[derive(Debug, PartialEq)]
pub struct Field {
    pub id: String,
    pub ty: Type,
}

pub trait DfinityInfo {
    fn ty() -> Type;
}

pub fn get_type<T>(_v: &T) -> Type where T: DfinityInfo {
    T::ty()
}

// ## Primitive Types

macro_rules! primitive_impl {
    ($t:ty, $id:tt) => {
        impl DfinityInfo for $t {
            fn ty() -> Type {
                Type::$id
            }
        }
    };
}

primitive_impl!(bool, Bool);
primitive_impl!(i8, Int);
primitive_impl!(i16, Int);
primitive_impl!(i32, Int);
primitive_impl!(i64, Int);
primitive_impl!(isize, Int);
primitive_impl!(u8, Nat);
primitive_impl!(u16, Nat);
primitive_impl!(u32, Nat);
primitive_impl!(u64, Nat);
primitive_impl!(usize, Nat);

impl<T> DfinityInfo for Option<T> where T: DfinityInfo {
    fn ty() -> Type { Type::Opt(Box::new(T::ty())) }
}

impl<T> DfinityInfo for Box<T> where T: DfinityInfo {
    fn ty() -> Type { T::ty() }
}
