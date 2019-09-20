
use super::error::{Error, Result};
use serde::Deserialize;
use serde::de;

use std::io;
/*
pub fn from_bytes<'a,T>(bytes: &'a [u8]) -> Result<T>
where T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_bytes(bytes);
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.is_empty() {
        Ok(t)
    } else {
        Err(Error::TrailingCharacters)
    }
}
*/
pub struct Deserializer<'de> {
    input: &'de [u8],
    table: Vec<Vec<u8>>,
}

impl<'de> Deserializer<'de> {
    pub fn from_bytes(input: &'de [u8]) -> Self {
        Deserializer {
            input: input,
            table: Vec::new(),
        }
    }
}

