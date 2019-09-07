extern crate dfx_derive;

pub use dfx_derive::*;

#[derive(Debug, PartialEq, Hash, Eq, Clone)]
pub enum Type {
    Null,
    Bool,
    Nat,
    Int,
    Text,
    Var(String),
    Opt(Box<Type>),
    Vec(Box<Type>),
    Record(Vec<Field>),
    Variant(Vec<Field>),
}

pub fn is_primitive(t: &Type) -> bool {
    use Type::*;
    match t {
        Null | Bool | Nat | Int | Text => true,
        Var(_) => true,
        Opt(_) | Vec(_) | Record(_) | Variant(_) => false,
    }
}

#[derive(Debug, PartialEq, Hash, Eq, Clone)]
pub struct Field {
    pub id: String,
    pub ty: Type,
}

pub trait DfinityInfo {
    fn ty() -> Type;
    fn name() -> Option<String> { None }
    fn _ty() -> Type {
        if let Some(var) = Self::name() {
            Type::Var(var)
        } else {
            Self::ty()
        }
    } 
}

pub fn get_type<T>(_v: &T) -> Type where T: DfinityInfo {
    T::ty()
}

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
primitive_impl!(String, Text);
primitive_impl!(&str, Text);

impl<T> DfinityInfo for Option<T> where T: DfinityInfo {
    fn ty() -> Type { Type::Opt(Box::new(T::_ty())) }
}

impl<T> DfinityInfo for Vec<T> where T: DfinityInfo {
    fn ty() -> Type { Type::Vec(Box::new(T::_ty())) }    
}

impl<T> DfinityInfo for [T] where T: DfinityInfo {
    fn ty() -> Type { Type::Vec(Box::new(T::_ty())) }    
}

impl<T,E> DfinityInfo for Result<T,E> where T: DfinityInfo, E: DfinityInfo {
    fn ty() -> Type {
        Type::Variant(vec![
            Field{ id: "Ok".to_owned(), ty: T::_ty() },
            Field{ id: "Err".to_owned(), ty: E::_ty() }]
        )
    }
}

impl<T> DfinityInfo for Box<T> where T: ?Sized + DfinityInfo {
    fn ty() -> Type { T::_ty() }
}

impl<'a,T> DfinityInfo for &'a T where T: 'a + ?Sized + DfinityInfo {
    fn ty() -> Type { T::_ty() }    
}
