extern crate paste;

use std::io::Read;
use super::error::{Error, Result};
use super::idl_hash;
use serde::de::{self, Visitor, DeserializeOwned};
use std::collections::{BTreeMap, VecDeque};

use leb128::read::{signed as sleb128_decode, unsigned as leb128_decode};

pub fn from_bytes<T>(bytes: &[u8]) -> Result<T>
where T: DeserializeOwned,
{
    let mut deserializer = Deserializer::from_bytes(bytes);
    deserializer.parse_table()?;
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.is_empty() && deserializer.current_type.is_empty() && deserializer.field_index.is_none() {
        Ok(t)
    } else {
        Err(Error::Message(format!("Trailing bytes: {:x?}, types: {:?}", deserializer.input, deserializer.current_type)))
    }
}

#[derive(Clone, Debug)]
enum RawValue { I(i64), U(u64) }
impl RawValue {
    fn get_i64(&self) -> Result<i64> {
        match *self {
            RawValue::I(i) => Ok(i),
            _ => Err(Error::Message("get_i64 fail".to_string())),
        }
    }
    fn get_u64(&self) -> Result<u64> {
        match *self {
            RawValue::U(u) => Ok(u),
            _ => Err(Error::Message("get_u64 fail".to_string())),
        }
    }
}

pub struct Deserializer<'de> {
    input: &'de [u8],
    table: Vec<Vec<RawValue>>,
    types: Vec<RawValue>,
    current_type: VecDeque<RawValue>,
    field_index: Option<&'static str>,
}

impl<'de> Deserializer<'de> {
    pub fn from_bytes(input: &'de [u8]) -> Self {
        Deserializer {
            input: input,
            table: Vec::new(),
            types: Vec::new(),
            // TODO consider borrowing
            current_type: VecDeque::new(),
            field_index: None,
        }
    }
    fn leb128_read(&mut self) -> Result<u64> {
        Ok(leb128_decode(&mut self.input).expect("Should read unsigned number"))
    }
    fn sleb128_read(&mut self) -> Result<i64> {
        Ok(sleb128_decode(&mut self.input).expect("Should read signed number"))
    }
    fn parse_string(&mut self, len: usize) -> Result<String> {
        let mut buf = Vec::new();
        buf.resize(len, 0);
        self.input.read_exact(&mut buf)?;
        Ok(String::from_utf8(buf).unwrap())
    }
    fn parse_char(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.input.read_exact(&mut buf)?;
        Ok(buf[0])
    }
    fn parse_magic(&mut self) -> Result<()> {
        let magic = self.parse_string(4)?;
        if magic == "DIDL" {
            Ok(())
        } else {
            Err(Error::Message(format!("wrong magic number {}", magic)))
        }
    }
    
    fn parse_table(&mut self) -> Result<()> {
        self.parse_magic()?;
        let len = self.leb128_read()?;
        for _i in 0..len {
            let mut buf = Vec::new();
            let ty = self.sleb128_read()?;
            buf.push(RawValue::I(ty));
            match ty {
                -18 | -19 => { // opt, vec
                    buf.push(RawValue::I(self.sleb128_read()?));
                },
                -20 | -21 => { //record, variant
                    let obj_len = self.leb128_read()?;
                    buf.push(RawValue::U(obj_len));
                    for _ in 0..obj_len {
                        buf.push(RawValue::U(self.leb128_read()?));
                        buf.push(RawValue::I(self.sleb128_read()?));
                    };
                },
                _ => {
                    return Err(Error::Message(format!("Unknown op_code {}", ty)))
                }
            };
            self.table.push(buf);
        };
        println!("{:?}", self.table);
        let len = self.leb128_read()?;
        for _i in 0..len {
            let ty = self.sleb128_read()?;
            self.types.push(RawValue::I(ty));
        };
        self.current_type.push_back(self.types[0].clone());
        println!("{:?}", self.types);
        Ok(())
    }

    fn parse_type(&mut self) -> Result<i64> {
        let op = self.current_type.pop_front().unwrap().get_i64()?;
        if op >= 0 {
            self.current_type.pop_front();
            let ty = &self.table[op as usize];
            for x in ty.iter().rev() {
                self.current_type.push_front(x.clone());
            }
            self.parse_type()
        } else {
            Ok(op)
        }
    }
}

macro_rules! primitive_impl {
    ($ty:ident, $opcode:literal, $method:ident $($cast:tt)*) => {
        paste::item! {
            fn [<deserialize_ $ty>]<V>(self, visitor: V) -> Result<V::Value>
            where V: Visitor<'de> {
                assert_eq!(self.parse_type().unwrap(), $opcode);
                visitor.[<visit_ $ty>](self.$method()? $($cast)*)
            }
        }
    };
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;
    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where V: Visitor<'de>
    {
        unimplemented!()
    }

    primitive_impl!(i8, -4, sleb128_read as i8);
    primitive_impl!(i16, -4, sleb128_read as i16);
    primitive_impl!(i32, -4, sleb128_read as i32);
    primitive_impl!(i64, -4, sleb128_read);
    primitive_impl!(u8, -3, leb128_read as u8);
    primitive_impl!(u16, -3, leb128_read as u16);
    primitive_impl!(u32, -3, leb128_read as u32);
    primitive_impl!(u64, -3, leb128_read);
    primitive_impl!(bool, -2, parse_char == 1u8);

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        assert_eq!(self.parse_type().unwrap(), -15);
        let len = self.leb128_read()? as usize;
        let value = self.parse_string(len)?;
        visitor.visit_string(value)         
    }
    
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        assert_eq!(self.parse_type().unwrap(), -15);        
        let len = self.leb128_read()? as usize;
        let value = std::str::from_utf8(&self.input[0..len]).unwrap();
        self.input = &self.input[len..];
        visitor.visit_borrowed_str(value)
    }
    
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        assert_eq!(self.parse_type().unwrap(), -18);
        let bit = self.parse_char()?;
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
        assert_eq!(self.parse_type().unwrap(), -1);
        visitor.visit_unit()
    }
    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }
    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value>
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
            -19 => {
                let len = self.leb128_read()? as u32;
                let value = visitor.visit_seq(Compound::new(&mut self, Style::Vector { len }));
                self.current_type.pop_front();
                value
            },
            -20 => {
                let len = self.current_type.pop_front().unwrap().get_u64()? as u32;
                visitor.visit_seq(Compound::new(&mut self, Style::Tuple { len, index: 0 }))
            },
            _ => {
                Err(Error::Message("seq only takes vector or tuple".to_string()))
            },
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
    where V: Visitor<'de>,
    {
        assert_eq!(self.parse_type().unwrap(), -20);
        let len = self.current_type.pop_front().unwrap().get_u64()? as u32;
        let mut fs = BTreeMap::new();
        for s in fields.iter() {
            assert_eq!(fs.insert(idl_hash(s), *s), None);
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
        assert_eq!(self.parse_type().unwrap(), -21);
        let len = self.current_type.pop_front().unwrap().get_u64()? as u32;
        let mut fs = BTreeMap::new();
        for s in variants.iter() {
            assert_eq!(fs.insert(idl_hash(s), *s), None);
        }
        let value = visitor.visit_enum(Compound::new(&mut self, Style::Enum{ len, fs }))?;
        Ok(value)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.field_index.is_none() {
            return Err(Error::Message("empty field_name".to_string()));
        }
        let v = visitor.visit_str(self.field_index.unwrap());
        self.field_index = None;
        v
    }
    
    serde::forward_to_deserialize_any! {
        char bytes byte_buf ignored_any f32 f64 map
    }
}

#[derive(Debug)]
enum Style {
    Tuple {len: u32, index: u32},
    Vector {len: u32},
    Struct {len: u32, fs: BTreeMap<u32, &'static str>},
    Enum {len: u32, fs: BTreeMap<u32, &'static str>},
}

struct Compound<'a, 'de: 'a> {
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
            Style::Tuple { ref len, ref mut index } => {
                if *index == *len {
                    return Ok(None);
                }
                let t_idx = self.de.current_type.pop_front().unwrap().get_u64()? as u32;
                assert_eq!(t_idx, *index);
                *index += 1;
                seed.deserialize(&mut *self.de).map(Some)
            },
            Style::Vector { ref mut len } => {
                if *len == 0 {
                    return Ok(None);
                }
                let ty = self.de.current_type.front().unwrap().clone();
                self.de.current_type.push_back(ty);
                *len -= 1;
                seed.deserialize(&mut *self.de).map(Some)
            },
            _ => Err(Error::Message("expect tuple".to_string()))
        }
    }
}

impl<'de, 'a> de::MapAccess<'de> for Compound<'a, 'de> {
    type Error = Error;
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where K: de::DeserializeSeed<'de> {
        match self.style {
            Style::Struct { ref mut len, ref fs } => {
                if *len == 0 {
                    return Ok(None);
                }
                *len -= 1;
                let hash = self.de.current_type.pop_front().unwrap().get_u64()? as u32;
                if self.de.field_index.is_some() {
                    return Err(Error::Message("field_name already taken".to_string()));
                }
                self.de.field_index = Some(fs[&hash]);
                seed.deserialize(&mut *self.de).map(Some)
            },
            _ => Err(Error::Message("expect struct".to_string()))
        }
    }
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where V: de::DeserializeSeed<'de> {
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
                    return Err(Error::Message(format!("variant index {} larger than length {}", index, len)));
                }
                for i in 0..len {
                    let hash = self.de.current_type.pop_front().unwrap().get_u64()? as u32;
                    let ty = self.de.current_type.pop_front().unwrap();
                    if i == index {
                        if self.de.field_index.is_some() {
                            return Err(Error::Message("field_index already taken".to_string()));
                        }                
                        self.de.field_index = Some(fs[&hash]);
                        // After we skip all the fields, ty will be the only thing left
                        self.de.current_type.push_back(ty);
                    }
                }        
                let val = seed.deserialize(&mut *self.de)?;
                Ok((val, self))
            },
            _ => Err(Error::Message("expect enum".to_string()))
        }
    }
}

impl<'de, 'a> de::VariantAccess<'de> for Compound<'a, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        assert_eq!(self.de.parse_type()?, -1);
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

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_struct(self.de, "_", fields, visitor)
    }
}
