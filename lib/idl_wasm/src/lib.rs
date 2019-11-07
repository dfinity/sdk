extern crate serde_idl;
extern crate wasm_bindgen;
extern crate serde_json;
extern crate js_sys;

use serde_idl::IDLArgs;
use serde_idl::value::IDLValue;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn encode(str: &str) -> Result<Vec<u8>, JsValue> {
    let args: IDLArgs = str.parse().map_err(|e:serde_idl::Error| JsValue::from_str(&e.to_string()))?;
    args.to_bytes().map_err(|e| JsValue::from_str(&e.to_string()))
}

fn to_idlvalue(val: &JsValue) -> Result<IDLValue, JsValue> {
    if js_sys::Number::is_integer(&val) {
        let v = val.as_f64().unwrap() as i64;
        Ok(IDLValue::Int(v))
    } else if let Some(v) = val.as_bool() {
        Ok(IDLValue::Bool(v))
    } else if val.is_object() {
        let iterator = js_sys::try_iter(val)?.ok_or_else(|| "Not iterable JS values")?;
        let mut vec = Vec::new();
        for x in iterator {
            let x = x?;
            let x = to_idlvalue(&x)?;
            vec.push(x);
        }
        Ok(IDLValue::Vec(vec))
    } else {
        Err(JsValue::from_str("Unknown type"))
    }
}

#[wasm_bindgen]
pub fn js_encode(val: &JsValue) -> Result<Vec<u8>, JsValue> {
    let mut idl = serde_idl::ser::IDLBuilder::new();
    let v = to_idlvalue(val)?;
    idl.value_arg(&v);
    idl.to_vec().map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn decode(bytes: &[u8]) -> Result<String, JsValue> {
    let args = IDLArgs::from_bytes(bytes).map_err(|e| JsValue::from_str(&e.to_string()))?;
    // Remove Serialize trait for IDLArgs when removing the following line.
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
