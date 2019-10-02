//! # Serde Dfinity IDL

extern crate dfx_info;
extern crate leb128;
extern crate num_enum;
extern crate serde;

pub use crate::de::IDLDeserialize;
pub use crate::error::{Error, Result};

pub mod de;
pub mod error;
pub mod ser;

#[macro_export]
macro_rules! Encode {
    ( $($x:expr),+ ) => {{
        let mut idl = serde_idl::ser::IDLBuilder::new();
        $(idl.arg($x);)+
        idl.to_vec().unwrap()
    }}
}

#[macro_export]
macro_rules! Decode {
    ( $hex:expr, $($name:ident: $ty:ty),+ ) => {
        let mut de = serde_idl::de::IDLDeserialize::new($hex);
        $(let $name: $ty = de.get_value().unwrap();)+
        de.done().unwrap();
    }
}

// IDL hash function is specified in
// https://github.com/dfinity-lab/actorscript/blob/master/design/IDL.md#shorthand-symbolic-field-ids
// which comes from https://caml.inria.fr/pub/papers/garrigue-polymorphic_variants-ml98.pdf
pub fn idl_hash(id: &str) -> u32 {
    let mut s: u32 = 0;
    for c in id.chars() {
        s = s.wrapping_mul(223).wrapping_add(c as u32);
    }
    s
}
