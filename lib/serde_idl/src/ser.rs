//! Serialize a Rust data structure to Dfinity IDL

use super::error::{Error, Result};
use serde::ser::{self, Impossible, Serialize};

use std::io;
use std::vec::Vec;
use leb128;
use std::collections::HashMap;
use dfx_info::Type;

use leb128::write::{signed as sleb128_encode, unsigned as leb128_encode};

/// Serializes a value to a vector.
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
where
    T: ser::Serialize + dfx_info::DfinityInfo,
{
    let mut vec = Vec::new();
    to_writer(&mut vec, value)?;
    Ok(vec)
}

/// Serializes a value to a writer.
pub fn to_writer<W, T>(mut writer: W, value: &T) -> Result<()>
where
    W: io::Write,
    T: ser::Serialize + dfx_info::DfinityInfo,
{
    writer.write_all(b"DIDL")?;
    
    let mut type_ser = TypeSerialize::new();
    let ty = T::ty();
    type_ser.serialize(&ty)?;
    writer.write_all(&type_ser.result)?;
    
    let mut value_ser = ValueSerializer::new();
    value.serialize(&mut value_ser)?;
    writer.write_all(&value_ser.value)?;
    Ok(())
}

/// A structure for serializing Rust values to IDL.
#[derive(Debug)]
pub struct ValueSerializer {
    value: Vec<u8>,
}

impl ValueSerializer
{
    /// Creates a new IDL serializer.
    #[inline]
    pub fn new() -> Self {
        ValueSerializer {
            value: Vec::new()
        }
    }

    fn write_sleb128(&mut self, value: i64) -> () {
        leb128::write::signed(&mut self.value, value).expect("should write signed number");
    }
    fn write_leb128(&mut self, value: u64) -> () {
        leb128::write::unsigned(&mut self.value, value).expect("should write signed number");
    }
}

impl<'a> ser::Serializer for &'a mut ValueSerializer
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Impossible<(), Error>;
    type SerializeTuple = Impossible<(), Error>;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = Impossible<(), Error>;
    type SerializeStruct = Compound<'a>;
    type SerializeStructVariant = Impossible<(), Error>;
    
    #[inline]
    fn serialize_bool(self, value: bool) -> Result<()> {
        let value = if value { 1 } else { 0 };
        Ok(self.write_sleb128(value))
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
        Ok(self.write_sleb128(value))
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
        Ok(self.write_leb128(value))
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
        Ok(self.write_leb128(0))
    }

    fn serialize_some<T: ?Sized>(self, v: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.write_leb128(1);
        v.serialize(self)
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
        Ok(Compound {ser: self, fields: Vec::new()})
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

pub struct Compound<'a> {
    ser: &'a mut ValueSerializer,
    fields: Vec<(&'static str, Vec<u8>)>,
}

impl<'a> ser::SerializeStruct for Compound<'a>
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        let mut ser = ValueSerializer::new();
        value.serialize(&mut ser)?;
        self.fields.push((key, ser.value));
        Ok(())
    }

    #[inline]
    fn end(mut self) -> Result<()> {
        self.fields.sort_unstable_by_key(|(id,_)| idl_hash(id));        
        for (_, mut buf) in self.fields {
            self.ser.value.append(&mut buf);
        };
        Ok(())
    }    
}

/// A structure for serializing Rust values to IDL types.
#[derive(Debug)]
pub struct TypeSerialize {
    type_table: Vec<Vec<u8>>,
    type_map: HashMap<Type, i32>,
    result: Vec<u8>,
}

#[inline]
fn idl_hash(id: &str) -> u32 {
    let mut s: u32 = 0;
    for c in id.chars() {
        s = s.wrapping_mul(223).wrapping_add(c as u32);
    }
    s
}

impl TypeSerialize
{
    #[inline]
    pub fn new() -> Self {
        TypeSerialize {
            type_table: Vec::new(),
            type_map: HashMap::new(),
            result: Vec::new()
        }
    }

    #[inline]
    fn build_type(&mut self, t: &Type) -> Result<()> {
        if !dfx_info::is_primitive(t) && !self.type_map.contains_key(t) {
            match t {
                Type::Opt(ref ty) => {
                    self.build_type(ty)?;
                    let mut buf = Vec::new();
                    sleb128_encode(&mut buf, -18)?;
                    self.encode(&mut buf, ty)?;
                    // add_type
                    let idx = self.type_table.len();            
                    self.type_map.insert((*t).clone(), idx as i32);            
                    self.type_table.push(buf);
                },
                /*Type::Record(fs) => {
                    let mut buf = sleb128_encode(-20);
                    leb128::write::unsigned(&mut buf, fs.len() as u64)?;
                    for (id, t) in fs {
                        let mut id_buf = leb128_encode(*id as u64);
                        let mut t_buf = self.encode(&t)?;
                        buf.append(&mut id_buf);
                        buf.append(&mut t_buf);
                    };
                    Ok(buf)
                },*/
                _ => ()
            };
        };
        Ok(())
    }

    fn encode(&mut self, buf: &mut Vec<u8>, t: &Type) -> Result<()> {
        match t {
            Type::Bool => sleb128_encode(buf, -2),
            Type::Nat => sleb128_encode(buf, -3),
            Type::Int => sleb128_encode(buf, -4),
            _ => {
                let idx = self.type_map.get(&t).expect("type not found");
                sleb128_encode(buf, *idx as i64)
            },
        }?;
        Ok(())
    }

    fn serialize(&mut self, t: &Type) -> Result<()> {
        self.build_type(t)?;
        println!("{:?}", self.type_map);

        leb128_encode(&mut self.result, self.type_table.len() as u64)?;
        self.result.append(&mut self.type_table.concat());
        let mut ty_encode = Vec::new();        
        self.encode(&mut ty_encode, t)?;
        self.result.append(&mut ty_encode);
        Ok(())
    }
}
/*
impl<'a> ser::Serializer for &'a mut TypeSerializer
{
    type Ok = Type;
    type Error = Error;

    type SerializeStruct = TypeCompound<'a>;
    type SerializeStructVariant = Impossible<Self::Ok, Error>;

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(TypeCompound {ser: self, fields: Vec::new()})
    }
}

pub struct TypeCompound<'a> {
    ser: &'a mut TypeSerializer,
    fields: Vec<(&'static str, Type)>,
}

impl<'a> ser::SerializeStruct for TypeCompound<'a>
{
    type Ok = Type;
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        let t = value.serialize(&mut *self.ser)?;
        self.fields.push((key, t));
        Ok(())
    }

    #[inline]
    fn end(mut self) -> Result<Type> {
        self.fields.sort_unstable_by_key(|(id,_)| idl_hash(id));
        let mut fs = Vec::new();
        for (k, t) in self.fields {
            fs.push((idl_hash(k), Box::new(t)));
        };
        let t = Type::Record(fs);
        self.ser.add_type(&t)?;
        Ok(t)
    }    
}
*/
