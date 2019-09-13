extern crate dfx_derive;
pub use dfx_derive::*;

pub mod types;
use types::{Type, TypeId};

pub trait DfinityInfo {
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
}
