use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::{collections::HashMap, fmt::Display, path::Path};

use crate::lib::error::ExtensionError;

pub static MANIFEST_FILE_NAME: &str = "extension.json";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ExtensionManifest {
    pub name: String,
    pub version: String,
    pub homepage: String,
    pub authors: Option<String>,
    pub summary: String,
    pub categories: Vec<String>,
    pub keywords: Option<Vec<String>>,
    pub description: Option<String>,
    pub subcommands: JsonValue, // TODO: awaiting https://dfinity.atlassian.net/browse/SDK-599
    pub dependencies: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtensionSubcommand {
    pub subcommands: Vec<ExtensionSubcommand>,
    pub key: String,
    pub summary: String,
}

impl Display for ExtensionManifest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Ok(s) = serde_json::to_string_pretty(self) else {
            return Err(std::fmt::Error)
        };
        write!(f, "{}", s)
    }
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
