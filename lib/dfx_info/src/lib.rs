extern crate dfx_derive;
pub use dfx_derive::*;

use std::collections::HashMap;
use std::cell::RefCell;

pub type TypeId = std::any::TypeId;

#[derive(Debug, PartialEq, Hash, Eq, Clone)]
pub enum Type {
    Null,
    Bool,
    Nat,
    Int,
    Text,
    Knot(TypeId),
    Unknown,
    Opt(Box<Type>),
    Vec(Box<Type>),
    Record(Vec<Field>),
    Variant(Vec<Field>),
}

pub fn is_primitive(t: &Type) -> bool {
    use Type::*;
    match t {
        Null | Bool | Nat | Int | Text => true,
        Unknown => panic!("Unknown type"),
        Knot(_) => true,
        Opt(_) | Vec(_) | Record(_) | Variant(_) => false,
    }
}

#[derive(Debug, PartialEq, Hash, Eq, Clone)]
pub struct Field {
    pub id: String,
    pub ty: Type,
}

pub trait DfinityInfo {
    fn ty() -> Type {
        let id = Self::id();
        if let Some(t) = find_type(&id) {
            match t {
                Type::Unknown => Type::Knot(id),
                _ => t,
            }
        } else {
            env_put(id, Type::Unknown);
            let t = Self::_ty();
            env_put(id, t.clone());
            t
        }
    }
    fn id() -> TypeId;
    fn _ty() -> Type;
}

pub fn find_type(id: &TypeId) -> Option<Type> {
    ENV.with(|e| {
        match e.borrow().get(id) {
            None => None,
            Some(t) => Some ((*t).clone()),
        }
    })
}

pub fn env_put(id: TypeId, t: Type) {
    ENV.with(|e| {
        drop(e.borrow_mut().insert(id, t))
    })
}

thread_local!{
    pub static ENV: RefCell<HashMap<TypeId, Type>> = RefCell::new(HashMap::new());
}

pub fn get_type<T>(_v: &T) -> Type where T: DfinityInfo {
    T::ty()
}

macro_rules! primitive_impl {
    ($t:ty, $id:tt) => {
        impl DfinityInfo for $t {
            fn id() -> TypeId { TypeId::of::<$t>() }            
            fn _ty() -> Type { Type::$id }
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


impl<T> DfinityInfo for Option<T> where T: DfinityInfo + 'static {
    fn id() -> TypeId { TypeId::of::<Option<T>>() }
    fn _ty() -> Type { Type::Opt(Box::new(T::ty())) }
}

impl<T> DfinityInfo for Vec<T> where T: DfinityInfo + 'static {
    fn id() -> TypeId { TypeId::of::<Vec<T>>() }        
    fn _ty() -> Type { Type::Vec(Box::new(T::ty())) }    
}

impl<T> DfinityInfo for [T] where T: DfinityInfo + 'static {
    fn id() -> TypeId { TypeId::of::<[T]>() }
    fn _ty() -> Type { Type::Vec(Box::new(T::ty())) }    
}

impl<T,E> DfinityInfo for Result<T,E> where T: DfinityInfo + 'static, E: DfinityInfo + 'static {
    fn id() -> TypeId { TypeId::of::<Result<T,E>>() }
    fn _ty() -> Type {
        Type::Variant(vec![
            Field{ id: "Ok".to_owned(), ty: T::ty() },
            Field{ id: "Err".to_owned(), ty: E::ty() }]
        )
    }
}

impl<T> DfinityInfo for Box<T> where T: ?Sized + DfinityInfo + 'static {
    fn id() -> TypeId { TypeId::of::<Box<T>>() }
    fn _ty() -> Type { T::ty() }
}


