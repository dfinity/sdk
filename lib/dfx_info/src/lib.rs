extern crate dfx_derive;
pub use dfx_derive::*;

use std::collections::HashMap;
use std::cell::RefCell;

// This is a re-implementation of std::any::TypeId to get rid of 'static constraint.
// The current TypeId doesn't consider lifetime while computing the hash, which is
// totally fine for IDL type, as we don't care about lifetime at all.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct TypeId { id: usize }
impl TypeId {
    pub fn of<T: ?Sized>() -> Self {
        TypeId { id: TypeId::of::<T> as usize }
    }
}

thread_local!{
    static ENV: RefCell<HashMap<TypeId, Type>> = RefCell::new(HashMap::new());
}

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

pub fn unroll(t: &Type) -> Type {
    use Type::*;
    match t {
        Knot(id) => find_type(id).unwrap(),
        Opt(ref t) => Opt(Box::new(unroll(t))),
        Vec(ref t) => Opt(Box::new(unroll(t))),
        Record(fs) => Record(fs.iter().map(|Field{id,ty}| {
            Field {id: id.to_string(), ty: unroll(ty)}
        }).collect()),
        Variant(fs) => Variant(fs.iter().map(|Field{id,ty}| {
            Field {id: id.to_string(), ty: unroll(ty)}
        }).collect()),        
        _ => (*t).clone(),
    }
}

#[derive(Debug, PartialEq, Hash, Eq, Clone)]
pub struct Field {
    pub id: String,
    pub ty: Type,
}

pub trait DfinityInfo {
    // memoized type derivation
    fn ty() -> Type {
        let id = Self::id();
        if let Some(t) = find_type(&id) {
            match t {
                Type::Unknown => Type::Knot(id),
                _ => t,
            }
        } else {
            env_add(id, Type::Unknown);
            let t = Self::_ty();
            env_add(id, t.clone());
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

pub fn show_env() {
    ENV.with(|e| println!("{:?}", e.borrow()));
}

fn env_add(id: TypeId, t: Type) {
    ENV.with(|e| {
        drop(e.borrow_mut().insert(id, t))
    })
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
primitive_impl!((), Null);


impl<T> DfinityInfo for Option<T> where T: DfinityInfo {
    fn id() -> TypeId { TypeId::of::<Option<T>>() }
    fn _ty() -> Type { Type::Opt(Box::new(T::ty())) }
}

impl<T> DfinityInfo for Vec<T> where T: DfinityInfo {
    fn id() -> TypeId { TypeId::of::<Vec<T>>() }        
    fn _ty() -> Type { Type::Vec(Box::new(T::ty())) }    
}

impl<T> DfinityInfo for [T] where T: DfinityInfo {
    fn id() -> TypeId { TypeId::of::<[T]>() }
    fn _ty() -> Type { Type::Vec(Box::new(T::ty())) }    
}

impl<T,E> DfinityInfo for Result<T,E> where T: DfinityInfo, E: DfinityInfo {
    fn id() -> TypeId { TypeId::of::<Result<T,E>>() }
    fn _ty() -> Type {
        Type::Variant(vec![
            Field{ id: "Ok".to_owned(), ty: T::ty() },
            Field{ id: "Err".to_owned(), ty: E::ty() }]
        )
    }
}

impl<T> DfinityInfo for Box<T> where T: ?Sized + DfinityInfo {
    fn id() -> TypeId { TypeId::of::<T>() } // ignore box
    fn _ty() -> Type { T::ty() }
}

impl<'a,T> DfinityInfo for &'a T where T: 'a + ?Sized + DfinityInfo {
    fn id() -> TypeId { TypeId::of::<&T>() } // ignore lifetime
    fn _ty() -> Type { T::ty() }
}


