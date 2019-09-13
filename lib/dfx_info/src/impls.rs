use IDLType;
use types::*;
use Serializer;

macro_rules! primitive_impl {
    ($t:ty, $id:tt, $method:ident $($cast:tt)*) => {
        impl IDLType for $t {
            fn id() -> TypeId { TypeId::of::<$t>() }            
            fn _ty() -> Type { Type::$id }
            fn serialize<S>(&self, serializer: S) -> Result<(), S::Error> where S: Serializer {
                serializer.$method(*self $($cast)*)
            }
        }
    };
}

primitive_impl!(bool, Bool, serialize_bool);
primitive_impl!(i8, Int, serialize_int as i64);
primitive_impl!(i16, Int, serialize_int as i64);
primitive_impl!(i32, Int, serialize_int as i64);
primitive_impl!(i64, Int, serialize_int);
primitive_impl!(isize, Int, serialize_int as i64);
primitive_impl!(u8, Nat, serialize_nat as u64);
primitive_impl!(u16, Nat, serialize_nat as u64);
primitive_impl!(u32, Nat, serialize_nat as u64);
primitive_impl!(u64, Nat, serialize_nat);
primitive_impl!(usize, Nat, serialize_nat as u64);
primitive_impl!(&str, Text, serialize_text);
primitive_impl!((), Null, serialize_null);

impl IDLType for String {
    fn id() -> TypeId { TypeId::of::<String>() }
    fn _ty() -> Type { Type::Text }
    fn serialize<S>(&self, serializer: S) -> Result<(), S::Error> where S: Serializer {
        serializer.serialize_text(self)
    }    
}

impl<T: Sized> IDLType for Option<T> where T: IDLType {
    fn id() -> TypeId { TypeId::of::<Option<T>>() }
    fn _ty() -> Type { Type::Opt(Box::new(T::ty())) }
    fn serialize<S>(&self, serializer: S) -> Result<(), S::Error> where S: Serializer {
        serializer.serialize_option(self.as_ref())
    }
}

impl<T> IDLType for Vec<T> where T: IDLType {
    fn id() -> TypeId { TypeId::of::<Vec<T>>() }        
    fn _ty() -> Type { Type::Vec(Box::new(T::ty())) }    
}

impl<T> IDLType for [T] where T: IDLType {
    fn id() -> TypeId { TypeId::of::<[T]>() }
    fn _ty() -> Type { Type::Vec(Box::new(T::ty())) }    
}

impl<T,E> IDLType for Result<T,E> where T: IDLType, E: IDLType {
    fn id() -> TypeId { TypeId::of::<Result<T,E>>() }
    fn _ty() -> Type {
        Type::Variant(vec![
            Field{ id: "Ok".to_owned(), ty: T::ty() },
            Field{ id: "Err".to_owned(), ty: E::ty() }]
        )
    }
}

impl<T> IDLType for Box<T> where T: ?Sized + IDLType {
    fn id() -> TypeId { TypeId::of::<T>() } // ignore box
    fn _ty() -> Type { T::ty() }
    fn serialize<S>(&self, serializer: S) -> Result<(), S::Error> where S: Serializer {
        (**self).serialize(serializer)
    }    
}

impl<'a,T> IDLType for &'a T where T: 'a + ?Sized + IDLType {
    fn id() -> TypeId { TypeId::of::<&T>() } // ignore lifetime
    fn _ty() -> Type { T::ty() }
    fn serialize<S>(&self, serializer: S) -> Result<(), S::Error> where S: Serializer {
        (**self).serialize(serializer)
    }    
}
