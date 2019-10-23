//! # Serde Dfinity IDL
//!
//! # Using the library
//!
//! ```
//! use serde_idl::{IDLType, Deserialize, Encode, Decode};
//! // Serialization
//! let bytes = Encode!(&[(42, "text")], &(42, "text"));
//! // Deserialization
//! Decode!(&bytes, a: Vec<(i64, &str)>, b: (i32, String));
//! assert_eq!(a, [(42, "text")]);
//! assert_eq!(b, (42i32, "text".to_string()));
//! ```
//!
//! # Serialize/Deserialize struct/enum
//!
//! ```
//! # #[macro_use] extern crate serde_idl; fn main() {
//! #[derive(IDLType, Deserialize)]
//! struct List {
//!     head: i32,
//!     tail: Option<Box<List>>,
//! }
//! let list = List { head: 42, tail: None };
//! let bytes = Encode!(&list);
//! Decode!(&bytes, l: List);
//! # }
//! ```

extern crate dfx_info;
extern crate leb128;
extern crate num_enum;
extern crate serde;

pub use crate::de::IDLDeserialize;
pub use crate::error::{Error, Result};
pub use dfx_info::IDLType;
pub use serde::Deserialize;

pub mod de;
pub mod error;
pub mod ser;
pub mod value;

pub const EMPTY_DIDL: &[u8] = b"DIDL\0\0";

/// Encode sequence of Rust values into IDL message.
#[macro_export]
macro_rules! Encode {
    ( $($x:expr),* ) => {{
        let mut idl = serde_idl::ser::IDLBuilder::new();
        $(idl.arg($x);)*
        idl.to_vec().unwrap()
    }}
}

/// Decode IDL message into a sequence of Rust values.
#[macro_export]
macro_rules! Decode {
    ( $hex:expr, $($name:ident: $ty:ty),* ) => {
        let mut de = serde_idl::de::IDLDeserialize::new($hex);
        $(let $name: $ty = de.get_value().unwrap();)*
        de.done().unwrap()
    }
}
