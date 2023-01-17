use schemars::JsonSchema;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub static COMMON_EXTENSIONS_MANIFEST_LOCATION: &str =
    "https://raw.githubusercontent.com/smallstepman/dfx-extensions/main/extensions-manifest.json";

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct ExtensionsManifest(pub HashMap<String, ExtensionSpec>);
type ExtensionName = String;
pub type ExtensionSpec = HashMap<ExtensionName, ExtensionLocation>;

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct ExtensionLocation {
    #[serde(rename = "downloadUrl")]
    pub download_url: String,
}

impl ExtensionsManifest {
    pub fn fetch() -> Result<ExtensionsManifest, Box<dyn std::error::Error>> {
        let resp = reqwest::blocking::get(COMMON_EXTENSIONS_MANIFEST_LOCATION)?;
        let extensions_manifest: ExtensionsManifest = resp.json()?;
        Ok(extensions_manifest)
    }

    pub fn find(&self, extension_name: &str, dfx_version: Version) -> Option<ExtensionLocation> {
        // this is very primitve and should be improved
        for (compat_dfx_version, manifests) in self.0.iter() {
            let sv_test = VersionReq::parse(compat_dfx_version);
            if sv_test.unwrap().matches(&dfx_version) {
                if let Some(extension_location) = manifests.get(extension_name) {
                    return Some(extension_location.clone());
                }
            }
        }
        println!("Extension not found");
        return None;
    }

    pub fn _list_compatible_extensions(&self) -> &'static str {
        todo!()
    }
}
