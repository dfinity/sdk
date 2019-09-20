use std::io::Read;
use super::error::{Error, Result};
use serde::Deserialize;
use serde::de;

use leb128::read::{signed as sleb128_decode, unsigned as leb128_decode};


pub fn from_bytes<'a>(bytes: &'a [u8]) -> Result<()>
{
    let mut deserializer = Deserializer::from_bytes(bytes);
    deserializer.parse_table()?;
    //let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.is_empty() {
        Ok(())
    } else {
        Err(Error::TrailingCharacters)
    }
}

pub struct Deserializer<'de> {
    input: &'de [u8],
    table: Vec<Vec<i64>>,
}

impl<'de> Deserializer<'de> {
    pub fn from_bytes(input: &'de [u8]) -> Self {
        Deserializer {
            input: input,
            table: Vec::new(),
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
        #[derive(Debug)]
        enum V { I(i64), U(u64) };
        let mut table: Vec<Vec<V>> = Vec::new();
        
        self.parse_magic()?;
        let len = self.leb128_read()?;
        for _i in 0..len {
            let mut buf = Vec::new();
            let ty = self.sleb128_read()?;
            buf.push(V::I(ty));
            match ty {
                -18 | -19 => { // opt, vec
                    buf.push(V::I(self.sleb128_read()?));
                },
                -20 | -21 => { //record, variant
                    let obj_len = self.leb128_read()?;
                    buf.push(V::U(obj_len));
                    for _ in 0..obj_len {
                        buf.push(V::U(self.leb128_read()?));
                        buf.push(V::I(self.sleb128_read()?));
                    };
                },
                _ => {
                    return Err(Error::Message(format!("Unknown op_code {}", ty)))
                }
            };
            table.push(buf);
        };
        
        panic!("{:?}", table);
        Ok(())
    }
}

#[test]
fn test() {
    from_bytes(&hex::decode("4449444c016e71").unwrap()).unwrap();
}
