//! # Serde Dfinity IDL

extern crate serde;
extern crate leb128;
extern crate dfx_info;

// Re-export the [items recommended by serde](https://serde.rs/conventions.html).
//#[doc(inline)]
//pub use crate::de::{from_str, Deserializer};
#[doc(inline)]
pub use crate::error::{Error, Result};
#[doc(inline)]
pub use crate::ser::{to_vec};

//pub mod de;
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
