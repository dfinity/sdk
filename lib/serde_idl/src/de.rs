extern crate paste;

use super::error::{Error, Result};
use super::idl_hash;
use num_enum::TryFromPrimitive;
use serde::de::{self, Visitor};
use std::collections::{BTreeMap, VecDeque};
use std::convert::TryFrom;
use std::io::Read;

use leb128::read::{signed as sleb128_decode, unsigned as leb128_decode};

const MAGIC_NUMBER: &[u8; 4] = b"DIDL";
#[derive(Debug, PartialEq, TryFromPrimitive)]
#[repr(i64)]
enum Opcode {
    Null = -1,
    Bool = -2,
    Nat = -3,
    Int = -4,
    Text = -15,
    Opt = -18,
    Vec = -19,
    Record = -20,
    Variant = -21,
}

pub struct IDLDeserialize<'de> {
    de: Deserializer<'de>,
}

impl<'de> IDLDeserialize<'de> {
    pub fn new(bytes: &'de [u8]) -> Self {
        let mut de = Deserializer::from_bytes(bytes);
        de.parse_table().unwrap();
        IDLDeserialize { de }
    }
    pub fn get_value<T>(&mut self) -> Result<T>
    where
        T: de::Deserialize<'de>,
    {
        let ty = self
            .de
            .types
            .pop_front()
            .ok_or(Error::Message("No more values to deserialize".to_string()))?;
        self.de.current_type.push_back(ty.clone());
        let v = T::deserialize(&mut self.de)?;
        if self.de.current_type.is_empty() && self.de.field_name.is_none() {
            Ok(v)
        } else {
            Err(Error::Message(format!(
                "Trailing types {:?}, field_name {:?}",
                self.de.current_type, self.de.field_name
            )))
        }
    }
    pub fn done(self) -> Result<()> {
        if !self.de.types.is_empty() {
            return Err(Error::Message(format!(
                "{} more values need to be deserialized",
                self.de.types.len()
            )));
        }
        if !self.de.input.is_empty() {
            return Err(Error::Message(format!(
                "Trailing bytes {:?}",
                self.de.input
            )));
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
enum RawValue {
    I(i64),
    U(u64),
}
impl RawValue {
    fn get_i64(&self) -> Result<i64> {
        match *self {
            RawValue::I(i) => Ok(i),
            _ => Err(Error::Message(format!("get_i64 fail: {:?}", *self))),
        }
    }
    fn get_u64(&self) -> Result<u64> {
        match *self {
            RawValue::U(u) => Ok(u),
            _ => Err(Error::Message(format!("get_u64 fail: {:?}", *self))),
        }
    }
}

pub struct Deserializer<'de> {
    input: &'de [u8],
    table: Vec<Vec<RawValue>>,
    types: VecDeque<RawValue>,
    current_type: VecDeque<RawValue>,
    field_name: Option<&'static str>,
}

impl<'de> Deserializer<'de> {
    pub fn from_bytes(input: &'de [u8]) -> Self {
        Deserializer {
            input,
            table: Vec::new(),
            types: VecDeque::new(),
            // TODO consider borrowing
            current_type: VecDeque::new(),
            field_name: None,
        }
    }
    fn error(&self, msg: &str) -> Error {
        Error::Message(msg.to_string())
    }
    fn error_states(&self, msg: &str) -> Error {
        let msg = format!(
            "{}. Trailing type {:?}, Trailing bytes {:?}",
            msg, self.current_type, self.input
        );
        Error::Message(msg)
    }
    fn leb128_read(&mut self) -> Result<u64> {
        leb128_decode(&mut self.input).map_err(|e| self.error_states(&e.to_string()))
    }
    fn sleb128_read(&mut self) -> Result<i64> {
        sleb128_decode(&mut self.input).map_err(|e| self.error_states(&e.to_string()))
    }
    fn parse_string(&mut self, len: usize) -> Result<String> {
        let mut buf = Vec::new();
        buf.resize(len, 0);
        self.input.read_exact(&mut buf)?;
        String::from_utf8(buf).map_err(|e| self.error(&e.to_string()))
    }
    fn parse_byte(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.input.read_exact(&mut buf)?;
        Ok(buf[0])
    }
    fn parse_magic(&mut self) -> Result<()> {
        let mut buf = [0u8; 4];
        self.input.read(&mut buf)?;
        if buf == *MAGIC_NUMBER {
            Ok(())
        } else {
            Err(self.error(&format!("wrong magic number {:?}", buf)))
        }
    }

    fn parse_table(&mut self) -> Result<()> {
        self.parse_magic()?;
        let len = self.leb128_read()?;
        for _i in 0..len {
            let mut buf = Vec::new();
            let ty = self.sleb128_read()?;
            buf.push(RawValue::I(ty));
            match Opcode::try_from(ty) {
                Ok(Opcode::Opt) | Ok(Opcode::Vec) => {
                    buf.push(RawValue::I(self.sleb128_read()?));
                }
                Ok(Opcode::Record) | Ok(Opcode::Variant) => {
                    let obj_len = self.leb128_read()?;
                    buf.push(RawValue::U(obj_len));
                    for _ in 0..obj_len {
                        buf.push(RawValue::U(self.leb128_read()?));
                        buf.push(RawValue::I(self.sleb128_read()?));
                    }
                }
                _ => return Err(self.error_states(&format!("Unsupported op_code {} in type table", ty))),
            };
            self.table.push(buf);
        }
        let len = self.leb128_read()?;
        for _i in 0..len {
            let ty = self.sleb128_read()?;
            self.types.push_back(RawValue::I(ty));
        }
        Ok(())
    }
    fn get_type_i64(&mut self) -> Result<i64> {
        self.current_type
            .pop_front()
            .ok_or(self.error("empty current_type"))?
            .get_i64()
            .map_err(|e| self.error_states(&e.to_string()))
    }

    fn parse_type(&mut self) -> Result<Opcode> {
        //let op = self.current_type.pop_front().unwrap().get_i64()?;
        let op = self.get_type_i64()?;
        if op >= 0 {
            self.current_type.pop_front();
            let ty = &self.table[op as usize];
            for x in ty.iter().rev() {
                self.current_type.push_front(x.clone());
            }
            self.parse_type()
        } else {
            match Opcode::try_from(op) {
                Ok(op) => Ok(op),
                Err(e) => Err(Error::Message(e.to_string())),
            }
        }
    }
    fn peek_type(&self) -> Result<Opcode> {
        let op = self.current_type.front().unwrap().get_i64()?;
        let ty = if op >= 0 {
            self.table[op as usize][0].get_i64()?
        } else {
            op
        };
        match Opcode::try_from(ty) {
            Ok(op) => Ok(op),
            Err(e) => Err(Error::Message(e.to_string())),
        }
    }
    fn check_type(&mut self, expected: Opcode) -> Result<()> {
        let wire_type = self.parse_type()?;
        if wire_type != expected {
            return Err(Error::Message(format!(
                "Type mismatch. Type on the wire: {:?}; Provided type: {:?}",
                wire_type, expected
            )));
        }
        Ok(())
    }
    fn set_field_name(&mut self, field: &'static str) -> Result<()> {
        if self.field_name.is_some() {
            return Err(Error::Message("field_name already taken".to_string()));
        }
        self.field_name = Some(field);
        Ok(())
    }
}

macro_rules! primitive_impl {
    ($ty:ident, $opcode:expr, $method:ident $($cast:tt)*) => {
        paste::item! {
            fn [<deserialize_ $ty>]<V>(self, visitor: V) -> Result<V::Value>
            where V: Visitor<'de> {
                self.check_type($opcode)?;
                visitor.[<visit_ $ty>](self.$method()? $($cast)*)
            }
        }
    };
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let t = self.peek_type()?;
        match t {
            Opcode::Int => self.deserialize_i64(visitor),
            Opcode::Nat => self.deserialize_u64(visitor),
            Opcode::Bool => self.deserialize_bool(visitor),
            Opcode::Text => self.deserialize_string(visitor),
            Opcode::Null => self.deserialize_unit(visitor),
            Opcode::Vec => self.deserialize_seq(visitor),
            Opcode::Opt => self.deserialize_option(visitor),
            Opcode::Record => self.deserialize_struct("_", &[], visitor),
            Opcode::Variant => self.deserialize_enum("_", &[], visitor),
        }
    }

    primitive_impl!(i8, Opcode::Int, sleb128_read as i8);
    primitive_impl!(i16, Opcode::Int, sleb128_read as i16);
    primitive_impl!(i32, Opcode::Int, sleb128_read as i32);
    primitive_impl!(i64, Opcode::Int, sleb128_read);
    primitive_impl!(u8, Opcode::Nat, leb128_read as u8);
    primitive_impl!(u16, Opcode::Nat, leb128_read as u16);
    primitive_impl!(u32, Opcode::Nat, leb128_read as u32);
    primitive_impl!(u64, Opcode::Nat, leb128_read);
    primitive_impl!(bool, Opcode::Bool, parse_byte == 1u8);

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.check_type(Opcode::Text)?;
        let len = self.leb128_read()? as usize;
        let value = self.parse_string(len)?;
        visitor.visit_string(value)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.check_type(Opcode::Text)?;
        let len = self.leb128_read()? as usize;
        let value = std::str::from_utf8(&self.input[0..len]).unwrap();
        self.input = &self.input[len..];
        visitor.visit_borrowed_str(value)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.check_type(Opcode::Opt)?;
        let bit = self.parse_byte()?;
        if bit == 0u8 {
            //self.parse_type() cannot be used as it will expand the type, which has no value
            self.current_type.pop_front();
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.check_type(Opcode::Null)?;
        visitor.visit_unit()
    }
    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }
    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }
    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.parse_type().unwrap() {
            Opcode::Vec => {
                let len = self.leb128_read()? as u32;
                let value = visitor.visit_seq(Compound::new(&mut self, Style::Vector { len }));
                self.current_type.pop_front();
                value
            }
            Opcode::Record => {
                let len = self
                    .current_type
                    .pop_front()
                    .ok_or(Error::Message("Cannot get length of record".to_string()))?
                    .get_u64()? as u32;
                visitor.visit_seq(Compound::new(&mut self, Style::Tuple { len, index: 0 }))
            }
            _ => Err(Error::Message("seq only takes vector or tuple".to_string())),
        }
    }
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }
    fn deserialize_struct<V>(
        mut self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.check_type(Opcode::Record)?;
        let len = self
            .current_type
            .pop_front()
            .ok_or(Error::Message("Cannot get length of record".to_string()))?
            .get_u64()? as u32;
        let mut fs = BTreeMap::new();
        for s in fields.iter() {
            if fs.insert(idl_hash(s), *s) != None {
                return Err(Error::Message(format!("hash collisiosn {}", s)));
            }
        }
        let value = visitor.visit_map(Compound::new(&mut self, Style::Struct { len, fs }))?;
        Ok(value)
    }

    fn deserialize_enum<V>(
        mut self,
        _name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.check_type(Opcode::Variant)?;
        let len = self
            .current_type
            .pop_front()
            .ok_or(Error::Message("Cannot get length of variant".to_string()))?
            .get_u64()? as u32;
        let mut fs = BTreeMap::new();
        for s in variants.iter() {
            if fs.insert(idl_hash(s), *s) != None {
                return Err(Error::Message(format!("hash collisiosn {}", s)));
            }
        }
        let value = visitor.visit_enum(Compound::new(&mut self, Style::Enum { len, fs }))?;
        Ok(value)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.field_name.is_none() {
            return Err(Error::Message("empty field_name".to_string()));
        }
        let v = visitor.visit_str(self.field_name.unwrap());
        self.field_name = None;
        v
    }

    serde::forward_to_deserialize_any! {
        char bytes byte_buf ignored_any f32 f64 map
    }
}

#[derive(Debug)]
enum Style {
    Tuple {
        len: u32,
        index: u32,
    },
    Vector {
        len: u32,
    },
    Struct {
        len: u32,
        fs: BTreeMap<u32, &'static str>,
    },
    Enum {
        len: u32,
        fs: BTreeMap<u32, &'static str>,
    },
}

struct Compound<'a, 'de> {
    de: &'a mut Deserializer<'de>,
    style: Style,
}

impl<'a, 'de> Compound<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, style: Style) -> Self {
        Compound { de, style }
    }
}

impl<'de, 'a> de::SeqAccess<'de> for Compound<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.style {
            Style::Tuple {
                ref len,
                ref mut index,
            } => {
                if *index == *len {
                    return Ok(None);
                }
                let t_idx = self.de.current_type.pop_front().unwrap().get_u64()? as u32;
                if t_idx != *index {
                    return Err(Error::Message(format!(
                        "Expect vector index {}, but get {}",
                        index, t_idx
                    )));
                }
                *index += 1;
                seed.deserialize(&mut *self.de).map(Some)
            }
            Style::Vector { ref mut len } => {
                if *len == 0 {
                    return Ok(None);
                }
                let ty = self.de.current_type.front().unwrap().clone();
                self.de.current_type.push_back(ty);
                *len -= 1;
                seed.deserialize(&mut *self.de).map(Some)
            }
            _ => Err(Error::Message("expect tuple".to_string())),
        }
    }
}

impl<'de, 'a> de::MapAccess<'de> for Compound<'a, 'de> {
    type Error = Error;
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        match self.style {
            Style::Struct {
                ref mut len,
                ref fs,
            } => {
                if *len == 0 {
                    return Ok(None);
                }
                *len -= 1;
                let hash = self.de.current_type.pop_front().unwrap().get_u64()? as u32;
                match fs.get(&hash) {
                    Some(field) => {
                        self.de.set_field_name(field)?;
                    }
                    None => {
                        // This triggers call to deserialize_any to skip both type and value of this unknown field.
                        self.de.set_field_name("_")?;
                    }
                }
                seed.deserialize(&mut *self.de).map(Some)
            }
            _ => Err(Error::Message("expect struct".to_string())),
        }
    }
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}

impl<'de, 'a> de::EnumAccess<'de> for Compound<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        match self.style {
            Style::Enum { len, ref fs } => {
                let index = self.de.leb128_read()? as u32;
                if index >= len {
                    return Err(Error::Message(format!(
                        "variant index {} larger than length {}",
                        index, len
                    )));
                }
                for i in 0..len {
                    let hash = self.de.current_type.pop_front().unwrap().get_u64()? as u32;
                    let ty = self.de.current_type.pop_front().unwrap();
                    if i == index {
                        match fs.get(&hash) {
                            Some(field) => {
                                self.de.set_field_name(field)?;
                            }
                            None => {
                                return Err(Error::Message(format!(
                                    "Unknown variant hash {}",
                                    hash
                                )))
                            }
                        }
                        // After we skip all the fields, ty will be the only thing left
                        self.de.current_type.push_back(ty);
                    }
                }
                let val = seed.deserialize(&mut *self.de)?;
                Ok((val, self))
            }
            _ => Err(Error::Message("expect enum".to_string())),
        }
    }
}

impl<'de, 'a> de::VariantAccess<'de> for Compound<'a, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        self.de.check_type(Opcode::Null)?;
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self.de, visitor)
    }

    fn struct_variant<V>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_struct(self.de, "_", fields, visitor)
    }
}
