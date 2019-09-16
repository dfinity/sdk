extern crate dfx_derive;
pub use dfx_derive::*;

pub mod types;
use types::{Type, TypeId};

mod impls;

pub trait IDLType {
    // memoized type derivation
    fn ty() -> Type {
        let id = Self::id();
        if let Some(t) = types::find_type(&id) {
            match t {
                Type::Unknown => Type::Knot(id),
                _ => t,
            }
        } else {
            types::env_add(id, Type::Unknown);
            let t = Self::_ty();
            types::env_add(id, t.clone());
            t
        }
    }
    fn id() -> TypeId;
    fn _ty() -> Type;
    // only serialize the value encoding
    fn idl_serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where S: Serializer { Ok(()) }
}

pub trait Serializer: Sized {
    type Error;
    type Compound : Compound<Error=Self::Error>;
    fn serialize_bool(self, v: bool) -> Result<(), Self::Error>;
    fn serialize_int(self, v: i64) -> Result<(), Self::Error>;
    fn serialize_nat(self, v: u64) -> Result<(), Self::Error>;
    fn serialize_text(self, v: &str) -> Result<(), Self::Error>;
    fn serialize_null(self, v:()) -> Result<(), Self::Error>;
    fn serialize_option<T: ?Sized>(self, v: Option<&T>) -> Result<(), Self::Error> where T: IDLType;
    fn serialize_compound(self) -> Result<Self::Compound, Self::Error>;
}

pub trait Compound {
    type Error;
    fn serialize_field<T: ?Sized>(&mut self, v: &T) -> Result<(), Self::Error> where T: IDLType;
}
