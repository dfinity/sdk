extern crate paste;

use std::io::Read;
use super::error::{Error, Result};
use super::idl_hash;
use serde::Deserialize;
use serde::de::{self, Visitor, DeserializeOwned};
use dfx_info::types::{Type, Field};
use std::collections::{HashMap, VecDeque};

use leb128::read::{signed as sleb128_decode, unsigned as leb128_decode};

pub fn from_bytes<T>(bytes: &[u8]) -> Result<T>
where T: DeserializeOwned,
{
    let mut deserializer = Deserializer::from_bytes(bytes);
    deserializer.parse_table()?;
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.is_empty() && deserializer.current_type.is_empty() {
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
    field_name: Option<&'static str>,
}

impl<'de> Deserializer<'de> {
    pub fn from_bytes(input: &'de [u8]) -> Self {
        Deserializer {
            input: input,
            table: Vec::new(),
            types: Vec::new(),
            // TODO consider borrowing
            current_type: VecDeque::new(),
            field_name: None,
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
        //let mut hashmap = HashMap::new();
        for _i in 0..len {
            let ty = self.sleb128_read()?;
            //let ty = self.build_type(&mut hashmap, &[RawValue::I(ty)])?;
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
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
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
            //self.parse_type()? cannot be used as it will expand the type, which has no value
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
        unimplemented!()
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
        unimplemented!()
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
    fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_struct<V>(
        mut self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where V: Visitor<'de>,
    {
        assert_eq!(self.parse_type().unwrap(), -20);
        let len = self.current_type.pop_front().unwrap().get_u64()?;
        
        println!("XX {} {:?}", name, fields);
        let value = visitor.visit_map(DeserializeMap::new(&mut self, len, fields))?;
        Ok(value)
    }
    
    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        println!("XX {} {:?}", name, variants);
        visitor.visit_bool(true)
    }
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = visitor.visit_str(self.field_name.unwrap());
        self.field_name = None;
        v
    }
    
    serde::forward_to_deserialize_any! {
        char bytes byte_buf ignored_any f32 f64
    }
}

struct DeserializeMap<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    len: u64,
    fs: HashMap<u32, &'static str>,
}

impl<'a, 'de> DeserializeMap<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, len: u64, fields: &'static [&'static str]) -> Self {
        let mut fs = HashMap::new();
        for s in fields.iter() {
            assert_eq!(fs.insert(idl_hash(s), *s), None);
        }
        DeserializeMap { de, len, fs }
    }
}

impl<'de, 'a> de::MapAccess<'de> for DeserializeMap<'a, 'de> {
    type Error = Error;
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where K: de::DeserializeSeed<'de> {
        if self.len == 0 {
            return Ok(None);
        }
        self.len -= 1;
        let hash = self.de.current_type.pop_front().unwrap().get_u64()? as u32;
        if self.de.field_name.is_some() {
            return Err(Error::Message("field_name already taken".to_string()));
        }
        self.de.field_name = Some(self.fs[&hash]);
        seed.deserialize(&mut *self.de).map(Some)            
    }
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where V: de::DeserializeSeed<'de> {
        seed.deserialize(&mut *self.de)
    }
}
