//! A canister metadata with key "dfx"
//!
//! The cli tool dfx should consolidate its usage of canister metadata into this single section
//! It's originally for pulling dependencies. But open to extend for other usage.
use crate::lib::error::DfxResult;
use anyhow::bail;
use dfx_core::config::model::dfinity::Pullable;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DfxMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pullable: Option<Pullable>,
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
