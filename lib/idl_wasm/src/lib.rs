extern crate serde_idl;
extern crate wasm_bindgen;

use serde_idl::IDLArgs;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn encode(str: &str) -> Vec<u8> {
    let args: IDLArgs = str.parse().unwrap();
    args.to_bytes().unwrap()
}

/*
#[wasm_bindgen]
pub fn decode(bytes: &[u8]) -> IDLArgs {
    IDLArgs::from_bytes(bytes).unwrap()
}*/

#[wasm_bindgen]
pub enum Type {
    Null,
    Bool,
    Nat,
    Int,
    Text,
    Opt,
    Vec,
    Record,
    Variant,
}

#[wasm_bindgen]
pub struct Value {
    pub ty: Type,
    pub bval: bool,
    pub nat: u64,
    pub int: i64,
    //pub opt: Option<Box<Value>>,
    pub vec: Vec<Value>,
    pub record: Vec<Field>,
    pub variant: Field,
}

#[wasm_bindgen]
pub struct Field {
    pub id: u32,
    pub val: Value,
}
