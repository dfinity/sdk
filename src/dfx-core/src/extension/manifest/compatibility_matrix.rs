use crate::error::extension::{
    FetchExtensionCompatibilityMatrixError,
    FetchExtensionCompatibilityMatrixError::{
        CompatibilityMatrixFetchError, MalformedCompatibilityMatrix,
    },
    FindLatestExtensionCompatibleVersionError,
    FindLatestExtensionCompatibleVersionError::{
        DfxVersionNotFoundInCompatibilityJson, ExtensionVersionNotFoundInRepository,
        ListOfVersionsForExtensionIsEmpty, MalformedVersionsEntryForExtensionInCompatibilityMatrix,
    },
};
use schemars::JsonSchema;
use semver::Version;
use serde::Deserialize;
use std::collections::HashMap;

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
    pub fn fetch() -> Result<Self, FetchExtensionCompatibilityMatrixError> {
        let resp = reqwest::blocking::get(COMMON_EXTENSIONS_MANIFEST_LOCATION).map_err(|e| {
            CompatibilityMatrixFetchError(COMMON_EXTENSIONS_MANIFEST_LOCATION.to_string(), e)
        })?;

        resp.json().map_err(MalformedCompatibilityMatrix)
    }

    pub fn find_latest_compatible_extension_version(
        &self,
        extension_name: &str,
        dfx_version: &Version,
    ) -> Result<Version, FindLatestExtensionCompatibleVersionError> {
        let manifests = self
            .0
            .get(dfx_version)
            .ok_or_else(|| DfxVersionNotFoundInCompatibilityJson(dfx_version.clone()))?;

        let extension_location = manifests.get(extension_name).ok_or_else(|| {
            ExtensionVersionNotFoundInRepository(
                extension_name.to_string(),
                dfx_version.clone(),
                COMMON_EXTENSIONS_MANIFEST_LOCATION.to_string(),
            )
        })?;
        let mut extension_versions = vec![];
        for ext_verion in extension_location.versions.iter().rev() {
            let version = Version::parse(ext_verion).map_err(|e| {
                MalformedVersionsEntryForExtensionInCompatibilityMatrix(ext_verion.to_string(), e)
            })?;
            extension_versions.push(version);
        }
        extension_versions.sort();
        extension_versions.reverse();
        extension_versions.first().cloned().ok_or_else(|| {
            ListOfVersionsForExtensionIsEmpty(
                COMMON_EXTENSIONS_MANIFEST_LOCATION.to_string(),
                dfx_version.clone(),
            )
        })
    }
}
