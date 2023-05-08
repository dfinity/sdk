use serde::Deserialize;
use std::{collections::HashMap, path::Path};

use crate::lib::error::ExtensionError;

pub static MANIFEST_FILE_NAME: &str = "extension.json";

#[derive(Debug, Deserialize)]
pub struct ExtensionManifest {
    pub name: String,
    pub version: String,
    pub homepage: String,
    pub authors: Option<String>,
    pub summary: String,
    pub categories: Vec<String>,
    pub keywords: Option<Vec<String>>,
    pub description: Option<String>,
    pub subcommands: serde_json::Value, // TODO: https://dfinity.atlassian.net/browse/SDK-599
    pub dependencies: Option<HashMap<String, String>>,
}

impl ExtensionManifest {
    pub fn new(name: &str, extensions_root_dir: &Path) -> Result<Self, ExtensionError> {
        let manifest_path = extensions_root_dir.join(name).join(MANIFEST_FILE_NAME);
        if !manifest_path.exists() {
            return Err(ExtensionError::ExtensionManifestMissing(name.to_owned()));
        }
        dfx_core::json::load_json_file(&manifest_path)
            .map_err(ExtensionError::ExtensionManifestIsNotValid)
    }
}
