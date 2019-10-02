use serde::ser;
use std::error::Error;
use std::fmt::{Display, Formatter};

/// Errors from reading a RequestId from a string. This is not the same as
/// deserialization.
#[derive(Debug)]
pub enum RequestIdFromStringError {
    InvalidSize(usize),
    FromHexError(hex::FromHexError),
}

/// An error during the calculation of the RequestId.
/// Since we use serde for serializing a data type into a hash, this has to support traits that
/// serde expects, such as Display
#[derive(Clone, Debug, PartialEq)]
pub enum RequestIdError {
    Custom(String),

    EmptySerializer,
    InvalidState,
    UnsupportedStructInsideStruct,

    // Base types.
    UnsupportedTypeBool,
    UnsupportedTypeU8,
    UnsupportedTypeU16,
    UnsupportedTypeU32,
    UnsupportedTypeU64,
    UnsupportedTypeU128,
    UnsupportedTypeI8,
    UnsupportedTypeI16,
    UnsupportedTypeI32,
    UnsupportedTypeI64,
    UnsupportedTypeI128,
    UnsupportedTypeF32,
    UnsupportedTypeF64,
    UnsupportedTypeChar,
    // UnsupportedTypeStr, // Supported
    UnsupportedTypeBytes,
    // UnsupportedTypeNone, // Supported
    // UnsupportedTypeSome, // Supported
    UnsupportedTypeUnit,
    UnsupportedTypePhantomData,

    // Variants and complex types.
    UnsupportedTypeUnitVariant,
    UnsupportedTypeNewtypeStruct(String),
    UnsupportedTypeNewTypeVariant,
    UnsupportedTypeSequence,
    UnsupportedTypeTuple,
    UnsupportedTypeTupleStruct,
    UnsupportedTypeTupleVariant,
    UnsupportedTypeMap,
    // UnsupportedTypeStruct, // Supported
    UnsupportedTypeStructVariant,
}

impl Display for RequestIdError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for RequestIdError {
    fn description(&self) -> &str {
        "An error happened during request_id."
    }
}

impl ser::Error for RequestIdError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        RequestIdError::Custom(msg.to_string())
    }
}
