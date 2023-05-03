//! A canister metadata with key "dfx"
//!
//! The cli tool dfx should consolidate its usage of canister metadata into this single section
//! It's originally for pulling dependencies. But open to extend for other usage.

use dfx_core::config::model::dfinity::PullReady;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DfxMetadata {
    pub pull_ready: PullReady,
}

impl DfxMetadata {
    pub fn set_pull_ready(&mut self, pull_ready: PullReady) {
        self.pull_ready = pull_ready;
    }
}
