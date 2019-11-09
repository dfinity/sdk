extern crate serde_idl;
extern crate wasm_bindgen;
extern crate js_sys;

use serde_idl::IDLArgs;
use serde_idl::value::IDLValue;
use js_sys::{Array};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen]
pub fn encode(str: &str) -> Result<Vec<u8>, JsValue> {
    let args: IDLArgs = str.parse().map_err(|e:serde_idl::Error| JsValue::from_str(&e.to_string()))?;
    args.to_bytes().map_err(|e| JsValue::from_str(&e.to_string()))
}

fn to_idlvalue(val: &JsValue) -> Result<IDLValue, JsValue> {
    if js_sys::Number::is_integer(&val) {
        let v = val.as_f64().unwrap() as i64;
        Ok(IDLValue::Int(v))
    } else if val.is_null() {
        Ok(IDLValue::Null)
    } else if let Some(v) = val.as_bool() {
        Ok(IDLValue::Bool(v))
    } else if let Some(v) = val.as_string() {
        // TODO use dyn_ref to avoid copying
        Ok(IDLValue::Text(v))
    } else if let Some(v) = val.dyn_ref::<Array>() {
        // TODO check if it's a tuple or vector
        let mut vec = Vec::new();
        for x in v.values() {
            let x = to_idlvalue(&x?)?;
            vec.push(x);
        }
        Ok(IDLValue::Vec(vec))
    } else {
        Err(JsValue::from_str("Unknown type"))
    }
}

fn to_jsvalue(val: &IDLValue) -> Result<JsValue, JsValue> {
    match *val {
        IDLValue::Null => Ok(JsValue::null()),
        IDLValue::Bool(b) => Ok(JsValue::from_bool(b)),
        IDLValue::Int(i) => Ok(JsValue::from_f64(i as f64)),
        IDLValue::Nat(n) => Ok(JsValue::from_f64(n as f64)),
        IDLValue::Text(ref s) => Ok(JsValue::from_str(s)),
        IDLValue::Vec(ref vec) => {
            let res = Array::new();
            for v in vec.iter() {
                let v = to_jsvalue(&v)?;
                res.push(&v);
            }
            Ok(res.unchecked_into::<JsValue>())
        },
        _ => Err(JsValue::from_str("Unsupported type"))
    }
}

#[wasm_bindgen]
pub fn js_encode(vals: Box<[JsValue]>) -> Result<Vec<u8>, JsValue> {
    let mut idl = serde_idl::ser::IDLBuilder::new();
    for v in vals.iter() {
        let v = to_idlvalue(v)?;
        idl.value_arg(&v);
    }
    idl.to_vec().map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn js_decode(bytes: &[u8]) -> Result<Array, JsValue> {
    let mut de = serde_idl::de::IDLDeserialize::new(bytes);
    let args = Array::new();
    while !de.is_done() {
        let v = de.get_value::<IDLValue>().map_err(|e| e.to_string())?;
        let v = to_jsvalue(&v)?;
        args.push(&v);
    }
    de.done().map_err(|e| e.to_string())?;
    Ok(args)
}


