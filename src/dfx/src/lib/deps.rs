use std::collections::BTreeMap;

use anyhow::bail;
use candid::Principal;
use serde::{Deserialize, Serialize};

use super::error::DfxResult;

#[derive(Serialize, Deserialize, Default)]
pub struct PulledJson {
    pub named: BTreeMap<String, Principal>,
    pub canisters: BTreeMap<Principal, PulledCanister>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct PulledCanister {
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

impl PulledJson {
    pub fn with_named(named: &BTreeMap<String, Principal>) -> Self {
        Self {
            named:named.clone(),
            ..Default::default()
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct InitJson {
    canisters: BTreeMap<Principal, InitItem>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct InitItem {
    // init argument in IDL string
    arg_str: Option<String>,
    // hex encoded bytes of init argument
    arg_raw: Option<String>,
}

impl InitJson {
    pub fn set_init_arg(
        &mut self,
        canister_id: Principal,
        arg_str: Option<String>,
        arg_raw: &[u8],
    ) {
        self.canisters.insert(
            canister_id,
            InitItem {
                arg_str,
                arg_raw: Some(hex::encode(arg_raw)),
            },
        );
    }

    pub fn set_empty_init(&mut self, canister_id: Principal) {
        self.canisters.insert(canister_id, InitItem::default());
    }

    pub fn contains(&self, canister_id: &Principal) -> bool {
        self.canisters.contains_key(canister_id)
    }

    pub fn get_arg_raw(&self, canister_id: &Principal) -> DfxResult<Vec<u8>> {
        match self.canisters.get(canister_id) {
            Some(item) => match &item.arg_raw {
                Some(s) => Ok(hex::decode(s)?),
                None => Ok(vec![]),
            },
            None => bail!(
                "Failed to find {0} entry in init.json. Please run `dfx deps init {0}`.",
                canister_id
            ),
        }
    }
}
