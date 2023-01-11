use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub static COMMON_EXTENSIONS_MANIFEST_LOCATION: &str =
    "https://raw.githubusercontent.com/smallstepman/dfx-extensions/main/extensions-manifest.json";

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct ExtensionsManifest(pub HashMap<String, ExtensionSpec>);
type ExtensionName = String;
type ExtensionSpec = HashMap<ExtensionName, ExtensionLocation>;

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct ExtensionLocation {
    #[serde(rename = "downloadUrl")]
    pub download_url: String,
}

pub fn fetch_dfinity_extension_manifest() -> Result<ExtensionsManifest, Box<dyn std::error::Error>>
{
    let resp = reqwest::blocking::get(COMMON_EXTENSIONS_MANIFEST_LOCATION)?;
    let extensions_manifest: ExtensionsManifest = resp.json()?;
    Ok(extensions_manifest)
}
