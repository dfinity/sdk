use dfx_info::types::{Field, Type};
use serde::de;
use serde::de::{Deserialize, Visitor};
use std::fmt;
use std::ops::Deref;

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
pub struct IDLField {
    pub id: u32,
    pub val: IDLValue,
}

impl dfx_info::IDLType for IDLValue {
    fn ty() -> Type {
        unreachable!();
    }
    fn id() -> dfx_info::types::TypeId {
        unreachable!();
    }
    fn _ty() -> Type {
        unreachable!();
    }
    fn value_ty(&self) -> Type {
        match *self {
            IDLValue::Null => Type::Null,
            IDLValue::Bool(_) => Type::Bool,
            IDLValue::Int(_) => Type::Int,
            IDLValue::Nat(_) => Type::Nat,
            IDLValue::Text(_) => Type::Text,
            IDLValue::Opt(ref v) => {
                let t = v.deref().value_ty();
                Type::Opt(Box::new(t))
            }
            IDLValue::Vec(ref vec) => {
                let t = if vec.is_empty() {
                    Type::Null
                } else {
                    vec[0].value_ty()
                };
                Type::Vec(Box::new(t))
            }
            IDLValue::Record(ref vec) => {
                let fs: Vec<_> = vec
                    .iter()
                    .map(|IDLField { id, val }| Field {
                        id: id.to_string(),
                        hash: *id,
                        ty: val.value_ty(),
                    })
                    .collect();
                Type::Record(fs)
            }
            IDLValue::Variant(ref v) => {
                let f = Field {
                    id: v.id.to_string(),
                    hash: v.id,
                    ty: v.val.value_ty(),
                };
                Type::Variant(vec![f])
            }
        }
    }
    fn idl_serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: dfx_info::Serializer,
    {
        use dfx_info::Compound;
        match *self {
            IDLValue::Null => serializer.serialize_null(()),
            IDLValue::Bool(b) => serializer.serialize_bool(b),
            IDLValue::Int(i) => serializer.serialize_int(i),
            IDLValue::Nat(n) => serializer.serialize_nat(n),
            IDLValue::Text(ref s) => serializer.serialize_text(s),
            IDLValue::Opt(ref v) => serializer.serialize_option(Some(v.deref())),
            IDLValue::Vec(ref vec) => {
                let mut ser = serializer.serialize_vec(vec.len())?;
                for e in vec.iter() {
                    ser.serialize_element(&e)?;
                }
                Ok(())
            }
            IDLValue::Record(ref vec) => {
                let mut ser = serializer.serialize_struct()?;
                for f in vec.iter() {
                    ser.serialize_element(&f.val)?;
                }
                Ok(())
            }
            IDLValue::Variant(ref v) => {
                let mut ser = serializer.serialize_variant(0)?;
                ser.serialize_element(&v.val)?;
                Ok(())
            }
        }
    }
}

impl<'de> Deserialize<'de> for IDLValue {
    fn deserialize<D>(deserializer: D) -> Result<IDLValue, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
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
            where
                E: serde::de::Error,
            {
                self.visit_string(String::from(value))
            }
            fn visit_none<E>(self) -> Result<IDLValue, E> {
                Ok(IDLValue::Null)
            }
            fn visit_some<D>(self, deserializer: D) -> Result<IDLValue, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let v = Deserialize::deserialize(deserializer)?;
                Ok(IDLValue::Opt(Box::new(v)))
            }
            fn visit_unit<E>(self) -> Result<IDLValue, E> {
                Ok(IDLValue::Null)
            }
            fn visit_seq<V>(self, mut visitor: V) -> Result<IDLValue, V::Error>
            where
                V: de::SeqAccess<'de>,
            {
                let mut vec = Vec::new();
                while let Some(elem) = visitor.next_element()? {
                    vec.push(elem);
                }
                Ok(IDLValue::Vec(vec))
            }
            fn visit_map<V>(self, mut visitor: V) -> Result<IDLValue, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mut vec = Vec::new();
                while let Some((key, value)) = visitor.next_entry()? {
                    if let IDLValue::Nat(hash) = key {
                        let f = IDLField {
                            id: hash as u32,
                            val: value,
                        };
                        vec.push(f);
                    } else {
                        unreachable!()
                    }
                }
                Ok(IDLValue::Record(vec))
            }
            fn visit_enum<V>(self, data: V) -> Result<IDLValue, V::Error>
            where
                V: de::EnumAccess<'de>,
            {
                use serde::de::VariantAccess;
                let (variant, visitor) = data.variant::<IDLValue>()?;
                if let IDLValue::Nat(hash) = variant {
                    //let val = visitor.struct_variant(&[], self)?;
                    visitor.unit_variant()?;
                    let f = IDLField {
                        id: hash as u32,
                        val: IDLValue::Null,
                    };
                    Ok(IDLValue::Variant(Box::new(f)))
                } else {
                    unreachable!()
                }
            }
        }

        deserializer.deserialize_any(IDLValueVisitor)
    }
}
