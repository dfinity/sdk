extern crate dfx_derive;

pub use dfx_derive::*;

#[derive(Debug, PartialEq)]
pub enum Type {
    Bool,
    Nat,
    Int,
    Opt(Box<Type>),
    Record(Vec<(u32, Box<Type>)>),
}

pub trait DfinityInfo {
    fn get_type(&self) -> Type;
}
