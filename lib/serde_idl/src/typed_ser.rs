//! Serialize a Rust data structure to Dfinity IDL

use super::error::{Error, Result};

use super::value::IDLValue;
use super::types::{IDLType, PrimType, Env};

use std::collections::HashMap;
use std::io;
use std::vec::Vec;

use leb128::write::{signed as sleb128_encode, unsigned as leb128_encode};

#[derive(Default)]
pub struct IDLBuilder {
    ser: Serializer,
}

impl IDLBuilder {
    pub fn new() -> Self {
        IDLBuilder {
            ser: Serializer::new(),
        }
    }
    pub fn arg<'a>(&'a mut self, ty: &IDLType, value: &IDLValue) -> Result<&'a mut Self> {
        self.ser.push_type(ty)?;
        ty.idl_serialize(&mut self.ser, value)?;
        Ok(self)
    }
    pub fn serialize<W: io::Write>(&mut self, mut writer: W) -> Result<()> {
        writer.write_all(b"DIDL")?;
        self.ser.serialize()?;
        writer.write_all(&self.ser.result)?;
        Ok(())
    }
    pub fn serialize_to_vec(&mut self) -> Result<Vec<u8>> {
        let mut vec = Vec::new();
        self.serialize(&mut vec)?;
        Ok(vec)
    }
}

impl IDLType {
    fn idl_serialize(&self, ser: Serializer, value: &IDLValue) -> Result<()> {
        match self {
            IDLType::PrimT(ref p) => p.idl_serialize(&mut ser, value),
            IDLType::VarT(ref id) => {
                let t = ser.env.get(id).unwrap();
                t.idl_serialize(&mut ser, value)
            },
            _ => unimplemented!(),
        }
    }
}

impl PrimType {
    fn idl_serialize(&self, ser: Serializer, value: &IDLValue) -> Result<()> {
        match self {
            PrimType::Nat => if IDLValue::Nat(n) = value {
                ser.serialize_nat(n)
            } else {
                panic!("not nat")
            }
            PrimType::Int => if IDLValue::Int(i) = value {
            } else {
                panic!("not int")
            }
            _ => unimplemented!(),
        }
    }
}

/// A structure for serializing Rust values to IDL.
#[derive(Debug, Default)]
pub struct ValueSerializer {
    value: Vec<u8>,
}

impl ValueSerializer {
    /// Creates a new IDL serializer.
    #[inline]
    pub fn new() -> Self {
        ValueSerializer { value: Vec::new() }
    }

    fn write_sleb128(&mut self, value: i64) {
        sleb128_encode(&mut self.value, value).unwrap();
    }
    fn write_leb128(&mut self, value: u64) {
        leb128_encode(&mut self.value, value).unwrap();
    }
}

impl<'a> dfx_info::Serializer for &'a mut ValueSerializer {
    type Error = Error;
    type Compound = Compound<'a>;
    fn serialize_bool(self, v: bool) -> Result<()> {
        let v = if v { 1 } else { 0 };
        self.write_leb128(v);
        Ok(())
    }
    fn serialize_int(self, v: i64) -> Result<()> {
        self.write_sleb128(v);
        Ok(())
    }
    fn serialize_nat(self, v: u64) -> Result<()> {
        self.write_leb128(v);
        Ok(())
    }
    fn serialize_text(self, v: &str) -> Result<()> {
        let mut buf = Vec::from(v.as_bytes());
        self.write_leb128(buf.len() as u64);
        self.value.append(&mut buf);
        Ok(())
    }
    fn serialize_null(self, _v: ()) -> Result<()> {
        Ok(())
    }
    fn serialize_option<T: ?Sized>(self, v: Option<&T>) -> Result<()>
    where
        T: dfx_info::IDLType,
    {
        match v {
            None => {
                self.write_leb128(0);
                Ok(())
            }
            Some(v) => {
                self.write_leb128(1);
                v.idl_serialize(self)
            }
        }
    }
    fn serialize_variant(self, index: u64) -> Result<Self::Compound> {
        self.write_leb128(index);
        Ok(Self::Compound { ser: self })
    }
    fn serialize_struct(self) -> Result<Self::Compound> {
        Ok(Self::Compound { ser: self })
    }
    fn serialize_vec(self, len: usize) -> Result<Self::Compound> {
        self.write_leb128(len as u64);
        Ok(Self::Compound { ser: self })
    }
}

pub struct Compound<'a> {
    ser: &'a mut ValueSerializer,
}
impl<'a> dfx_info::Compound for Compound<'a> {
    type Error = Error;
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: dfx_info::IDLType,
    {
        value.idl_serialize(&mut *self.ser)?;
        Ok(())
    }
}

/// A structure for serializing Rust values to IDL types.
#[derive(Debug, Default)]
pub struct Serializer {
    type_table: Vec<Vec<u8>>,
    type_map: HashMap<IDLType, i32>,
    type_args: Vec<IDLType>,
    values: Vec<u8>,
    env: Env,
    result: Vec<u8>,
}

impl Serializer {
    #[inline]
    pub fn new(env: Env) -> Self {
        Serializer {
            type_table: Vec::new(),
            type_map: HashMap::new(),
            type_args: Vec::new(),
            values: Vec::new(),
            env: HashMap::new(),
            result: Vec::new(),
        }
    }

    #[inline]
    fn build_type(&mut self, t: &IDLType) -> Result<()> {
        if let IDLType::PrimT(_) = t {
            return Ok(());
        }
        if self.type_map.contains_key(t) {
            return Ok(());
        }
        
        //let idx = self.type_table.len();
        self.type_map.insert((*t).clone(), -1);
        //self.type_table.push(Vec::new());
        let mut buf = Vec::new();
        match t {
            Type::Opt(ref ty) => {
                self.build_type(ty)?;
                sleb128_encode(&mut buf, -18)?;
                self.encode(&mut buf, ty)?;
            }
            Type::Vec(ref ty) => {
                self.build_type(ty)?;
                sleb128_encode(&mut buf, -19)?;
                self.encode(&mut buf, ty)?;
            }
            Type::Record(fs) => {
                for Field { ty, .. } in fs.iter() {
                    self.build_type(ty)?;
                }
                
                sleb128_encode(&mut buf, -20)?;
                leb128_encode(&mut buf, fs.len() as u64)?;
                for Field { hash, ty, .. } in fs.iter() {
                    leb128_encode(&mut buf, u64::from(*hash))?;
                    self.encode(&mut buf, ty)?;
                }
            }
            Type::Variant(fs) => {
                for Field { ty, .. } in fs.iter() {
                    self.build_type(ty)?;
                }
                
                sleb128_encode(&mut buf, -21)?;
                leb128_encode(&mut buf, fs.len() as u64)?;
                for Field { hash, ty, .. } in fs.iter() {
                    leb128_encode(&mut buf, u64::from(*hash))?;
                    self.encode(&mut buf, ty)?;
                }
            }
            _ => panic!("unreachable"),
        };
        self.type_table[idx] = buf;
        Ok(())
    }

    fn push_type(&mut self, t: &IDLType) -> Result<()> {
        self.args.push(t.clone());
        self.build_type(t)
    }

    fn encode(&self, buf: &mut Vec<u8>, t: &IDLType) -> Result<()> {
        match t {
            Type::Null => sleb128_encode(buf, -1),
            Type::Bool => sleb128_encode(buf, -2),
            Type::Nat => sleb128_encode(buf, -3),
            Type::Int => sleb128_encode(buf, -4),
            Type::Text => sleb128_encode(buf, -15),
            Type::Knot(id) => {
                let ty = dfx_info::types::find_type(*id).expect("knot TypeId not found");
                let idx = self
                    .type_map
                    .get(&ty)
                    .unwrap_or_else(|| panic!("knot type {:?} not found", ty));
                sleb128_encode(buf, i64::from(*idx))
            }
            _ => {
                let idx = self
                    .type_map
                    .get(&t)
                    .unwrap_or_else(|| panic!("type {:?} not found", t));
                sleb128_encode(buf, i64::from(*idx))
            }
        }?;
        Ok(())
    }

    fn serialize(&mut self) -> Result<()> {
        leb128_encode(&mut self.result, self.type_table.len() as u64)?;
        self.result.append(&mut self.type_table.concat());

        leb128_encode(&mut self.result, self.args.len() as u64)?;
        let mut ty_encode = Vec::new();
        for t in self.args.iter() {
            self.encode(&mut ty_encode, t)?;
        }
        self.result.append(&mut ty_encode);
        Ok(())
    }
}
