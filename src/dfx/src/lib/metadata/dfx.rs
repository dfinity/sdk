//! A canister metadata with key "dfx"
//!
//! The cli tool dfx should consolidate its usage of canister metadata into this single section
//! It's originally for pulling dependencies. But open to extend for other usage.
use crate::lib::error::DfxResult;
use anyhow::{bail, Context};
use candid::Principal;
use dfx_core::config::model::dfinity::PullableConfig;
use dfx_core::fs::read_to_string;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
    pub fn set_pullable(&mut self, pullable_config: PullableConfig) -> DfxResult {
        let wasm_url_str = match (pullable_config.wasm_url, pullable_config.dynamic_wasm_url) {
            (Some(_), Some(_)) => {
                bail!("Cannot define `wasm_url` and `dynamic_wasm_url` at the same time.")
            }
            (None, None) => {
                bail!("Pullable canister must define `wasm_url` or `dynamic_wasm_url`.")
            }
            (Some(s), None) => s,
            (None, Some(dwu)) => {
                let path = PathBuf::from(dwu.path);
                let file_content = read_to_string(&path)?;
                file_content
            }
        };

        reqwest::Url::parse(&wasm_url_str).with_context(|| {
            format!(
                "Failed to set wasm_url: \"{}\" is not a valid URL.",
                wasm_url_str
            )
        })?;

        self.pullable = Some(Pullable {
            wasm_url: wasm_url_str,
            wasm_hash: None, // TODO: get from config
            dependencies: pullable_config.dependencies,
            init_guide: pullable_config.init_guide,
        });
        Ok(())
    }

    pub fn get_pullable(&self) -> DfxResult<&Pullable> {
        match &self.pullable {
            Some(pullable) => Ok(pullable),
            None => bail!("The `dfx` metadata doesn't contain the `pullable` object."),
        }
    }
}
