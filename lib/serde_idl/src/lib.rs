//! # Serde Dfinity IDL

extern crate serde;
extern crate leb128;
extern crate dfx_info;

// Re-export the [items recommended by serde](https://serde.rs/conventions.html).
//#[doc(inline)]
pub use crate::de::{from_bytes, Deserializer};
pub use crate::error::{Error, Result};

pub mod de;
pub mod error;
#[macro_use]
pub mod ser;

#[macro_export]
macro_rules! IDL {
    ( $($x:expr),+ ) => {{
        let mut idl = serde_idl::ser::IDLBuilder::new();
        $(idl.arg($x);)+
        idl.to_vec().unwrap()
    }}
}

pub fn idl_hash(id: &str) -> u32 {
    let mut s: u32 = 0;
    for c in id.chars() {
        s = s.wrapping_mul(223).wrapping_add(c as u32);
    }
    s
}
