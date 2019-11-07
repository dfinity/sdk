extern crate serde_idl;
extern crate wasm_bindgen;
extern crate serde_json;

use serde_idl::IDLArgs;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn encode(str: &str) -> Result<Vec<u8>, JsValue> {
    let args: IDLArgs = str.parse().map_err(|e:serde_idl::Error| JsValue::from_str(&e.to_string()))?;
    args.to_bytes().map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn js_encode(val: &JsValue) -> Result<Vec<u8>, JsValue> {
}

#[wasm_bindgen]
pub fn decode(bytes: &[u8]) -> Result<String, JsValue> {
    let args = IDLArgs::from_bytes(bytes).map_err(|e| JsValue::from_str(&e.to_string()))?;
    // JsValue::from_serde(&args).unwrap()
    serde_json::to_string(&args).map_err(|e| JsValue::from_str(&e.to_string()))
}

/*
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
*/
