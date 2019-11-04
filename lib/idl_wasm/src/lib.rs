extern crate wasm_bindgen;
extern crate serde_idl;

use wasm_bindgen::prelude::*;
use serde_idl::IDLArgs;

#[wasm_bindgen]
pub fn encode(str: &str) -> Vec<u8> {
    let args: IDLArgs = str.parse().unwrap();
    args.to_bytes().unwrap()
}
