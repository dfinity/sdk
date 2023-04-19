use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::{collections::HashMap, fmt::Display, path::PathBuf};

use crate::lib::error::ExtensionError;

pub static MANIFEST_FILE_NAME: &str = "extension.toml";

#[derive(Debug, Deserialize, Serialize)]
struct Extension {
    extension: ExtensionManifest,
}

#[derive(Debug, Deserialize, Serialize)]
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

impl Display for ExtensionManifest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Ok(json) = serde_json::to_string_pretty(self) else {
            return Err(std::fmt::Error)
        };
        write!(f, "{}", json)
    }
}

impl ExtensionManifest {
    pub fn from_extension_directory(path: PathBuf) -> Result<Self, ExtensionError> {
        let manifest_path = path.join(MANIFEST_FILE_NAME);
        let ext: Extension = toml::from_str(&dfx_core::fs::read_to_string(&manifest_path)?)
            .map_err(|e| ExtensionError::ExtensionAlreadyInstalled(e.to_string()))?;
        Ok(ext.extension)
    }
}
