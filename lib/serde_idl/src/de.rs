use std::io::Read;
use super::error::{Error, Result};
use serde::Deserialize;
use serde::de;
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

#[test]
fn test() {
    //from_bytes(&hex::decode("4449444c016c02d3e3aa027e868eb7027c0100012a").unwrap()).unwrap();    
    //from_bytes(&hex::decode("4449444c026e016e7c010001012a").unwrap()).unwrap();
    from_bytes(&hex::decode("4449444c026d016c02007c0171020001012a04746578742a0474657874").unwrap()).unwrap();
    from_bytes(&hex::decode("4449444c026e016c02a0d2aca8047c90eddae70400010000").unwrap()).unwrap();
}
