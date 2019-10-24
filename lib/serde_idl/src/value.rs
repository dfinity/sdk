use serde::de;
use serde::de::{Deserialize, Visitor};
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum IDLValue {
    Bool(bool),
    Null,
    Text(String),
    Int(i64),
    Nat(u64),
    Opt(Box<IDLValue>),
    Vec(Vec<IDLValue>),
    Record(Vec<IDLField>),
    Variant(Box<IDLField>),
}

#[derive(Debug, PartialEq)]
pub struct IDLField {
    pub id: u32,
    pub val: IDLValue,
}

impl<'de> Deserialize<'de> for IDLValue {
    fn deserialize<D>(deserializer: D) -> Result<IDLValue, D::Error>
    where D: serde::Deserializer<'de> {
        struct IDLValueVisitor;
        
        impl<'de> Visitor<'de> for IDLValueVisitor {
            type Value = IDLValue;
            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("any valid IDL value")
            }
            fn visit_bool<E>(self, value: bool) -> Result<IDLValue, E> {
                Ok(IDLValue::Bool(value))
            }
            fn visit_i64<E>(self, value: i64) -> Result<IDLValue, E> {
                Ok(IDLValue::Int(value.into()))
            }
            fn visit_u64<E>(self, value: u64) -> Result<IDLValue, E> {
                Ok(IDLValue::Nat(value.into()))
            }
            fn visit_string<E>(self, value: String) -> Result<IDLValue, E> {
                Ok(IDLValue::Text(value))
            }
            fn visit_str<E>(self, value: &str) -> Result<IDLValue, E>
            where E: serde::de::Error {
                self.visit_string(String::from(value))
            }
            fn visit_none<E>(self) -> Result<IDLValue, E> {
                Ok(IDLValue::Null)
            }
            fn visit_some<D>(self, deserializer: D) -> Result<IDLValue, D::Error>
            where D: serde::Deserializer<'de> {
                let v = Deserialize::deserialize(deserializer)?;
                Ok(IDLValue::Opt(Box::new(v)))
            }
            fn visit_unit<E>(self) -> Result<IDLValue, E> {
                Ok(IDLValue::Null)
            }
            fn visit_seq<V>(self, mut visitor: V) -> Result<IDLValue, V::Error>
            where V: de::SeqAccess<'de> {
                let mut vec = Vec::new();
                while let Some(elem) = visitor.next_element()? {
                    vec.push(elem);
                }
                Ok(IDLValue::Vec(vec))
            }
            fn visit_map<V>(self, mut visitor: V) -> Result<IDLValue, V::Error>
            where V: de::MapAccess<'de> {
                let mut vec = Vec::new();
                while let Some((key, value)) = visitor.next_entry()? {
                    if let IDLValue::Nat(hash) = key {
                        let f = IDLField { id: hash as u32, val: value };
                        vec.push(f);
                    } else {
                        unreachable!()
                    }
                }
                Ok(IDLValue::Record(vec))
            }
            fn visit_enum<V>(self, data: V) -> Result<IDLValue, V::Error>
            where V: de::EnumAccess<'de> {
                use serde::de::VariantAccess;                
                let (variant, visitor) = data.variant::<IDLValue>()?;
                if let IDLValue::Nat(hash) = variant {
                    //let val = visitor.struct_variant(&[], self)?;
                    visitor.unit_variant()?;
                    let f = IDLField { id: hash as u32, val: IDLValue::Null };
                    Ok(IDLValue::Variant(Box::new(f)))
                } else {
                    unreachable!()
                }
            }
        }

        deserializer.deserialize_any(IDLValueVisitor)
    }
}

