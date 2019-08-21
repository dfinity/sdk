//! # Serde Dfinity IDL

//#[macro_use]
extern crate serde;
extern crate leb128;

// Re-export the [items recommended by serde](https://serde.rs/conventions.html).
//#[doc(inline)]
//pub use crate::de::{from_str, Deserializer};
#[doc(inline)]
pub use crate::error::{Error, Result};
#[doc(inline)]
pub use crate::ser::{to_vec, Serializer};

//#[macro_use]
//mod macros;

//pub mod de;
pub mod error;
pub mod ser;
//pub mod value;
