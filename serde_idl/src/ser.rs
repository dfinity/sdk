//! Serialize a Rust data structure to Dfinity IDL

use super::error::{Error, Result};
use serde::ser::{self, Impossible, Serialize};

use std::io;
use leb128;

/// Serializes a value to a vector.
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
where
    T: ser::Serialize,
{
    let mut vec = Vec::new();
    to_writer(&mut vec, value)?;
    Ok(vec)
}

/// Serializes a value to a writer.
pub fn to_writer<W, T>(mut writer: W, value: &T) -> Result<()>
where
    W: io::Write,
    T: ser::Serialize,
{
    writer.write_all(b"DIDL")?;
    let mut ser = Serializer::new(writer);
    value.serialize(&mut ser)
}

enum Type {
    Bool = -2,
    Nat = -3,
    Int = -4,
}
    
/// A structure for serializing Rust values to IDL.
#[derive(Debug)]
pub struct Serializer<W> {
    writer: W,
}

impl<W> Serializer<W>
where
    W: io::Write,
{
    /// Creates a new IDL serializer.
    ///
    /// `to_vec` and `to_writer` should normally be used instead of this method.
    #[inline]
    pub fn new(writer: W) -> Self {
        Serializer {
            writer: writer,
        }
    }

    /// Unwrap the `Writer` from the `Serializer`.
    #[inline]
    pub fn into_inner(self) -> W {
        self.writer
    }

    #[inline]
    fn write_sleb128(&mut self, value: i64) -> Result<()> {
        let mut buf = Vec::new();
        leb128::write::signed(&mut buf, value).expect("should write signed number");
        self.writer.write_all(&buf).map_err(|e| e.into())
    }
    
    #[inline]
    fn write_leb128(&mut self, value: u64) -> Result<()> {
        let mut buf = Vec::new();
        leb128::write::unsigned(&mut buf, value).expect("should write signed number");
        self.writer.write_all(&buf).map_err(|e| e.into())
    }

    #[inline]
    fn write_type(&mut self, t: Type) -> Result<()> {
        self.write_sleb128(t as i64)
    }
}

impl<'a, W> ser::Serializer for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Impossible<(), Error>;
    type SerializeTuple = Impossible<(), Error>;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = Impossible<(), Error>;
    type SerializeStruct = Impossible<(), Error>;
    type SerializeStructVariant = Impossible<(), Error>;
    
    #[inline]
    fn serialize_bool(self, value: bool) -> Result<()> {
        self.write_type(Type::Bool)?;
        let value = if value { 1 } else { 0 };
        self.writer.write_all(&[value]).map_err(|e| e.into())
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> Result<()> {
        self.serialize_i64(value as i64)
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> Result<()> {
        self.serialize_i64(value as i64)
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<()> {
        self.serialize_i64(value as i64)
    }

    #[inline]
    fn serialize_i64(self, value: i64) -> Result<()> {
        self.write_type(Type::Int)?;
        self.write_sleb128(value)
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<()> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> Result<()> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> Result<()> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> Result<()> {
        self.write_type(Type::Nat)?;
        self.write_leb128(value)
    }

    #[inline]
    fn serialize_f32(self, _v: f32) -> Result<()> {
        Err(Error::todo())
    }

    #[inline]
    fn serialize_f64(self, _v: f64) -> Result<()> {
        Err(Error::todo())
    }

    #[inline]
    fn serialize_char(self, _v: char) -> Result<()> {
        Err(Error::todo())
    }

    fn serialize_str(self, _v: &str) -> Result<()> {
        Err(Error::todo())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<()> {
        Err(Error::todo())
    }

    fn serialize_none(self) -> Result<()> {
        Err(Error::todo())
    }

    fn serialize_some<T: ?Sized>(self, _v: &T) -> Result<()>
    where
        T: Serialize,
    {
        Err(Error::todo())
    }

    fn serialize_unit(self) -> Result<()> {
        Err(Error::todo())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        Err(Error::todo())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        Err(Error::todo())
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<()>
    where
        T: Serialize,
    {
        Err(Error::todo())
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<()>
    where
        T: Serialize,
    {
        Err(Error::todo())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(Error::todo())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(Error::todo())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::todo())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::todo())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::todo())
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(Error::todo())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::todo())
    }    
}
