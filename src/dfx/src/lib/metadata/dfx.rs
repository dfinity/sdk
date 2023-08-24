//! A canister metadata with key "dfx"
//!
//! The cli tool dfx should consolidate its usage of canister metadata into this single section
//! It's originally for pulling dependencies. But open to extend for other usage.
use crate::lib::error::DfxResult;
use anyhow::bail;
use candid::Principal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DfxMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pullable: Option<Pullable>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Pullable {
    /// # wasm_url
    /// The Url to download canister wasm.
    pub wasm_url: String,

    /// # wasm_hash
    /// SHA256 hash of the wasm module located at wasm_url.
    /// Only define this if the on-chain canister wasm is expected not to match the wasm at wasm_url.
    pub wasm_hash: Option<String>,

    /// # dependencies
    /// Canister IDs (Principal) of direct dependencies.
    pub dependencies: Vec<Principal>,

    /// # init_guide
    /// A message to guide consumers how to initialize the canister.
    pub init_guide: String,
}

impl DfxMetadata {
    pub fn set_pullable(&mut self, pullable: Pullable) {
        self.pullable = Some(pullable);
    }

    pub fn get_pullable(&self) -> DfxResult<&Pullable> {
        match &self.pullable {
            Some(pullable) => Ok(pullable),
            None => bail!("The `dfx` metadata doesn't contain the `pullable` object."),
        }
    }
}
