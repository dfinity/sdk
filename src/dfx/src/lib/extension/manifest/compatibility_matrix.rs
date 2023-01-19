use itertools::Itertools;
use schemars::JsonSchema;
use semver::Version;
use serde::Deserialize;
use std::collections::HashMap;

use crate::lib::error::{DfxError, DfxResult, ExtensionError};

pub static COMMON_EXTENSIONS_MANIFEST_LOCATION: &str =
    "https://raw.githubusercontent.com/smallstepman/dfx-extensions/main/compatibility.json";
// pub static COMMON_EXTENSIONS_MANIFEST_LOCATION: &str =
//     "http://localhost:8000/Desktop/sdk_extensions/test/compatibility.json";

type DfxVersion = Version;
type ExtensionName = String;

#[derive(Deserialize, JsonSchema, Debug)]
pub struct ExtensionsCompatibilityMatrix(
    pub HashMap<DfxVersion, HashMap<ExtensionName, ExtensionCompatibleVersions>>,
);

#[derive(Deserialize, JsonSchema, Debug, Clone)]
pub struct ExtensionCompatibleVersions {
    pub versions: Vec<String>,
}

impl ExtensionsCompatibilityMatrix {
    pub fn fetch() -> DfxResult<Self> {
        let resp = reqwest::blocking::get(COMMON_EXTENSIONS_MANIFEST_LOCATION)?;
        let matrix: Self = resp.json()?;
        Ok(matrix)
    }

    pub fn find_latest_compatible_extension_version(
        &self,
        extension_name: &str,
        dfx_version: Version,
    ) -> DfxResult<Version> {
        let Some(manifests) = self.0.get(&dfx_version) else {
                    return Err(DfxError::new(ExtensionError::ExtensionError(format!(
            "can't find dfx version: {} in dfx extensions compatibility matrix: {}",
            dfx_version, COMMON_EXTENSIONS_MANIFEST_LOCATION
        ))));

        };
        let Some(extension_location) = manifests.get(extension_name) else{
            return Err(DfxError::new(ExtensionError::ExtensionError(
            format!("can't find extension: {} in dfx extensions compatibility matrix: {} for dfx version: {}", extension_name, dfx_version, COMMON_EXTENSIONS_MANIFEST_LOCATION)
        )));

        };
        return extension_location
            .versions
            .iter()
            .map(|v| Version::parse(v).unwrap_or(Version::new(0, 0, 0)))
            .sorted()
            .rev()
            .nth(0)
            .ok_or(DfxError::new(ExtensionError::ExtensionError(format!(
                "versions array empty for dfx version: {} and extension name: {}",
                dfx_version, extension_name
            ))));
    }

    pub fn _list_compatible_extensions(&self) -> &'static str {
        todo!()
    }
}
