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
    fn get_type(&self) -> Type;
}

// ## Primitive Types

macro_rules! primitive_impl {
    ($t:ty, $id:tt) => {
        impl DfinityInfo for $t {
            #[inline]
            fn get_type(&self) -> Type {
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

