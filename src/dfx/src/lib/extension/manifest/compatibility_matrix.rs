use schemars::JsonSchema;
use semver::Version;
use serde::Deserialize;
use std::collections::HashMap;

use crate::lib::error::ExtensionError;

pub static COMMON_EXTENSIONS_MANIFEST_LOCATION: &str =
    "https://raw.githubusercontent.com/dfinity/dfx-extensions/main/compatibility.json";

type DfxVersion = Version;
type ExtensionName = String;

#[derive(Deserialize, JsonSchema, Debug)]
pub struct ExtensionCompatibilityMatrix(
    pub HashMap<DfxVersion, HashMap<ExtensionName, ExtensionCompatibleVersions>>,
);

#[derive(Deserialize, JsonSchema, Debug, Clone)]
pub struct ExtensionCompatibleVersions {
    pub versions: Vec<String>,
}

impl ExtensionCompatibilityMatrix {
    pub fn fetch() -> Result<Self, ExtensionError> {
        let Ok(resp) = reqwest::blocking::get(COMMON_EXTENSIONS_MANIFEST_LOCATION) else {
            return Err(ExtensionError::CompatibilityMatrixFetchError(COMMON_EXTENSIONS_MANIFEST_LOCATION.to_string()));
        };
        resp.json()
            .map_err(|e| ExtensionError::MalformedCompatibilityMatrix(e))
    }

    pub fn find_latest_compatible_extension_version(
        &self,
        extension_name: &str,
        dfx_version: Version,
    ) -> Result<Version, ExtensionError> {
        let Some(manifests) = self.0.get(&dfx_version) else {
                    return Err(ExtensionError::DfxVersionNotFoundInCompatibilityJson(
            dfx_version, COMMON_EXTENSIONS_MANIFEST_LOCATION.to_string()
        ));

        };
        let Some(extension_location) = manifests.get(extension_name) else{
            return Err(ExtensionError::ExtensionVersionNotFoundInRepository(extension_name.to_string(), dfx_version.to_string(), COMMON_EXTENSIONS_MANIFEST_LOCATION.to_string()));
        };
        let mut extension_versions = vec![];
        for ext_verion in extension_location.versions.iter().rev() {
            let Ok(version) = Version::parse(ext_verion) else {
                return Err(ExtensionError::MalformedVersionsEntryForExtensionInCompatibilityMatrix(
                    ext_verion.to_string(),
                ));
            };
            extension_versions.push(version);
        }
        extension_versions.sort();
        extension_versions.reverse();
        extension_versions.first().cloned().ok_or_else(|| {
            ExtensionError::ListOfVersionsForExtensionIsEmpty(
                COMMON_EXTENSIONS_MANIFEST_LOCATION.to_string(),
            )
        })
    }
}
