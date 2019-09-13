//! Serialize a Rust data structure to Dfinity IDL

use super::error::{Error, Result};
use serde::ser::{self, Impossible, Serialize};

use std::io;
use std::vec::Vec;
use std::collections::HashMap;
use dfx_info::types::{Type, Field};

use leb128::write::{signed as sleb128_encode, unsigned as leb128_encode};

/// Serializes a value to a vector.
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
where
    T: ser::Serialize + dfx_info::IDLType,
{
    let mut vec = Vec::new();
    to_writer(&mut vec, value)?;
    Ok(vec)
}

/// Serializes a value to a writer.
pub fn to_writer<W, T>(mut writer: W, value: &T) -> Result<()>
where
    W: io::Write,
    T: ser::Serialize + dfx_info::IDLType,
{
    writer.write_all(b"DIDL")?;
    
    let mut type_ser = TypeSerialize::new();
    type_ser.serialize(&T::ty())?;
    writer.write_all(&type_ser.result)?;
    
    let mut value_ser = ValueSerializer::new();
    //value.serialize(&mut value_ser)?;
    //dfx_info::IDLType::serialize(&value, &mut value_ser)?;
    serde::Serialize::serialize(&value, &mut value_ser)?;
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
        sleb128_encode(&mut self.value, value).unwrap();
    }
    fn write_leb128(&mut self, value: u64) -> () {
        leb128_encode(&mut self.value, value).unwrap();
    }
}

impl dfx_info::Serializer for &mut ValueSerializer {
    type Error = Error;
    fn serialize_bool(self, v: bool) -> Result<()> {
        let v = if v { 1 } else { 0 };
        Ok(self.write_leb128(v))
    }
    fn serialize_int(self, v: i64) -> Result<()> {
        Ok(self.write_sleb128(v))
    }
    fn serialize_nat(self, v: u64) -> Result<()> {
        Ok(self.write_leb128(v))
    }
    fn serialize_text(self, v: &str) -> Result<()> {
        let mut buf = Vec::from(v.as_bytes());
        self.write_leb128(buf.len() as u64);
        self.value.append(&mut buf);
        Ok(())        
    }
    fn serialize_null(self, _v:()) -> Result<()> {
        Ok(())
    }
    fn serialize_option<T: ?Sized>(self, v: Option<&T>) -> Result<()>
    where T: dfx_info::IDLType {
        match v {
            None => Ok(self.write_leb128(0)),
            Some(v) => {
                self.write_leb128(1);
                v.serialize(self)
            }
        }
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
        Ok(self.write_leb128(value))
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

    fn serialize_str(self, v: &str) -> Result<()> {
        let mut buf = Vec::from(v.as_bytes());
        self.write_leb128(buf.len() as u64);
        self.value.append(&mut buf);
        Ok(())
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
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        // TODO update index according to idl_hash
        Ok(self.write_leb128(variant_index as u64))
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
        //self.fields.sort_unstable_by_key(|(id,_)| idl_hash(id));        
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


//fn sort_fields(fs: &Vec<Field>) -> Vec<(u32, &Type)> {
//    let fs: Vec<(u32, &Type)> =
//        fs.into_iter().map(|Field {id,hash,ty}| (hash.clone(), ty)).collect();
    //let unique_ids: BTreeSet<_> = fs.iter().map(|(hash,_)| hash).collect();
    //assert_eq!(unique_ids.len(), fs.len());    
    //fs.sort_unstable_by_key(|(id,_)| *id);
//    fs
//}

// TypeSerialize is implemented outside of the serde framework, as serde only supports value, not type.

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
        if !dfx_info::types::is_primitive(t) && !self.type_map.contains_key(t) {
            // This is a hack to remove (some) equivalent mu types
            // from the type table.
            // Someone should implement Pottier's O(nlogn) algorithm
            // http://gallium.inria.fr/~fpottier/publis/gauthier-fpottier-icfp04.pdf
            let unrolled = dfx_info::types::unroll(t);
            if let Some(idx) = self.type_map.get(&unrolled) {
                let idx = idx.clone();
                self.type_map.insert((*t).clone(), idx);
                return Ok(());
            }
            
            let idx = self.type_table.len();
            self.type_map.insert((*t).clone(), idx as i32);
            self.type_table.push(Vec::new());
            let mut buf = Vec::new();
            match t {
                Type::Opt(ref ty) => {
                    self.build_type(ty)?;
                    sleb128_encode(&mut buf, -18)?;
                    self.encode(&mut buf, ty)?;
                },
                Type::Vec(ref ty) => {
                    self.build_type(ty)?;
                    sleb128_encode(&mut buf, -19)?;
                    self.encode(&mut buf, ty)?;
                },                
                Type::Record(fs) => {
                    //let fs = sort_fields(fs);
                    for Field {id:_,hash:_,ty} in fs.iter() {
                        self.build_type(ty).unwrap();
                    };
                    
                    sleb128_encode(&mut buf, -20)?;
                    leb128_encode(&mut buf, fs.len() as u64)?;
                    for Field {id:_,hash,ty} in fs.iter() {
                        leb128_encode(&mut buf, *hash as u64)?;
                        self.encode(&mut buf, ty)?;
                    };
                },
                Type::Variant(fs) => {
                    //let fs = sort_fields(fs);
                    for Field{id:_,hash:_,ty} in fs.iter() {
                        self.build_type(ty).unwrap();
                    };
                    
                    sleb128_encode(&mut buf, -21)?;
                    leb128_encode(&mut buf, fs.len() as u64)?;
                    for Field{id:_,hash,ty} in fs.iter() {
                        leb128_encode(&mut buf, *hash as u64)?;
                        self.encode(&mut buf, ty)?;
                    };
                },                
                _ => panic!("unreachable"),
            };
            self.type_table[idx] = buf;
        };
        Ok(())
    }

    fn encode(&mut self, buf: &mut Vec<u8>, t: &Type) -> Result<()> {
        match t {
            Type::Null => sleb128_encode(buf, -1),
            Type::Bool => sleb128_encode(buf, -2),
            Type::Nat => sleb128_encode(buf, -3),
            Type::Int => sleb128_encode(buf, -4),
            Type::Text => sleb128_encode(buf, -15),
            Type::Knot(id) => {
                let ty = dfx_info::types::find_type(id)
                    .expect("knot TypeId not found");
                let idx = self.type_map.get(&ty)
                    .expect(&format!("knot type {:?} not found", ty));
                sleb128_encode(buf, *idx as i64)
            },
            _ => {
                let idx = self.type_map.get(&t)
                    .expect(&format!("type {:?} not found", t));
                sleb128_encode(buf, *idx as i64)
            },
        }?;
        Ok(())
    }

    fn serialize(&mut self, t: &Type) -> Result<()> {
        self.build_type(t)?;
        //println!("{:?}", self.type_map);

        leb128_encode(&mut self.result, self.type_table.len() as u64)?;
        self.result.append(&mut self.type_table.concat());
        let mut ty_encode = Vec::new();        
        self.encode(&mut ty_encode, t)?;
        self.result.append(&mut ty_encode);
        Ok(())
    }
}

