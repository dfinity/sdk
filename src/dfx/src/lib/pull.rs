use std::collections::HashMap;

use candid::Principal;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct PulledDirectory {
    pub canisters: HashMap<Principal, PulledCanister>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct PulledCanister {
    // Direct pulled dependency has a name defined in dfx.json
    pub name: Option<String>,
    // dfx:deps
    pub deps: Vec<Principal>,
    // dfx:wasm_url, once we can download wasm directly from IC, this field will be optional
    pub wasm_url: Option<String>,
    // the hash on chain
    // dfx:wasm_hash if defined
    // or get from canister_status
    pub wasm_hash: String,
    // dfx:init
    pub init: Option<String>,
}
