use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::{collections::HashMap, fmt::Display};

pub static MANIFEST_FILE_NAME: &str = "manifest.json";

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
    pub commands: JsonValue,
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
