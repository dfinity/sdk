use std::io::Read;
use super::error::{Error, Result};
use serde::Deserialize;
use serde::de::{self, Visitor};
use dfx_info::types::{Type, Field};
use std::collections::HashMap;

use leb128::read::{signed as sleb128_decode, unsigned as leb128_decode};

pub fn from_bytes<'a>(bytes: &'a [u8]) -> Result<()>
{
    let mut deserializer = Deserializer::from_bytes(bytes);
    deserializer.parse_table()?;
    //let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.is_empty() {
        Ok(())
    } else {
        Err(Error::Message(format!("Trailing bytes: {:x?}", deserializer.input)))
    }
}

#[derive(Debug)]
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
    types: Vec<Type>,
}

impl<'de> Deserializer<'de> {
    pub fn from_bytes(input: &'de [u8]) -> Self {
        Deserializer {
            input: input,
            table: Vec::new(),
            types: Vec::new(),
        }
    }
    fn leb128_read(&mut self) -> Result<u64> {
        Ok(leb128_decode(&mut self.input).expect("Should read unsigned number"))
    }
    fn sleb128_read(&mut self) -> Result<i64> {
        Ok(sleb128_decode(&mut self.input).expect("Should read signed number"))
    }
    fn parse_magic(&mut self) -> Result<()> {
        let mut buf = [0; 4];
        self.input.read_exact(&mut buf)?;
        let magic = String::from_utf8(buf.to_vec()).unwrap();
        if magic == "DIDL" {
            Ok(())
        } else {
            Err(Error::Message(format!("wrong magic number {:x?}", buf)))
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
        let mut hashmap = HashMap::new();
        for _i in 0..len {
            let ty = self.sleb128_read()?;
            let ty = self.build_type(&mut hashmap, &[RawValue::I(ty)])?;
            self.types.push(ty);
        };
        println!("{:?}", self.types);
        Ok(())
    }
    fn build_type(&self, hashmap: &mut HashMap<i64, Type>, ty: &[RawValue]) -> Result<Type> {
        if ty.is_empty() {
            return Err(Error::Message("empty ty".to_string()));
        }
        let t = ty[0].get_i64()?;
        match t {
            -1 => Ok(Type::Null),
            -2 => Ok(Type::Bool),
            -3 => Ok(Type::Nat),
            -4 => Ok(Type::Int),
            -15 => Ok(Type::Text),
            -18 => {
                let t1 = self.build_type(hashmap, &ty[1..])?;
                Ok(Type::Opt(Box::new(t1)))
            },
            -19 => {
                let t1 = self.build_type(hashmap, &ty[1..])?;
                Ok(Type::Vec(Box::new(t1)))
            },            
            -20 => {
                let mut iter = ty[1..].iter();
                let len = iter.next().unwrap().get_u64()?;
                let mut fs = Vec::new();
                for _i in 0..len {
                    let hash = iter.next().unwrap().get_u64()?;
                    let t1 = self.build_type(hashmap, iter.as_slice())?;
                    iter.next();
                    fs.push(Field {id: hash.to_string(), hash: hash as u32, ty: t1 });
                };
                Ok(Type::Record(fs))
            },
            _ if t >= 0 => {
                match hashmap.get(&t) {
                    Some(t1) => Ok(t1.clone()),
                    None => {
                        hashmap.insert(t, Type::Unknown);
                        let raw = &self.table[t as usize][..];
                        let t1 = self.build_type(hashmap, &raw)?;
                        hashmap.insert(t, t1.clone());
                        Ok(t1)
                    },
                }
            },
            _ => Err(Error::Message(format!("Unknown type {}", t)))
        }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where V: Visitor<'de>
    {
        match self.types[0] {
            Type::Bool => self.deserialize_bool(visitor),
            _ => Err(Error::Message("Unsupported type".to_string()))
        }
    }
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0; 1];
        self.input.read_exact(&mut buf)?;
        visitor.visit_bool(buf == [1])
    }
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.sleb128_read().unwrap() as i8)
    }
    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(self.sleb128_read().unwrap() as i16)
    }
    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.sleb128_read().unwrap() as i32)
    }
    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.sleb128_read()?)
    }
    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.leb128_read().unwrap() as u8)
    }
    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(self.leb128_read().unwrap() as u16)
    }
    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.leb128_read().unwrap() as u32)
    }
    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.leb128_read()?)
    }
    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }
    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
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
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }    
}

#[test]
fn test() {
    //from_bytes(&hex::decode("4449444c016c02d3e3aa027e868eb7027c0100012a").unwrap()).unwrap();    
    //from_bytes(&hex::decode("4449444c026e016e7c010001012a").unwrap()).unwrap();
    from_bytes(&hex::decode("4449444c026d016c02007c0171020001012a04746578742a0474657874").unwrap()).unwrap();
    from_bytes(&hex::decode("4449444c026e016c02a0d2aca8047c90eddae70400010000").unwrap()).unwrap();
}
