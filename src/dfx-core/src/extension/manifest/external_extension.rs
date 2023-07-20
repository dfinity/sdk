use crate::error::extension::ExtensionError;

use super::{ExtensionCompatibilityMatrix, ExtensionManifest};

use reqwest::Url;
use semver::Version;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct ExternalExtensionManifest {
    pub compatibility: ExtensionCompatibilityMatrix,
    pub extensions: HashMap<String, HashMap<Version, ExtensionManifest>>,
}

impl ExternalExtensionManifest {
    pub fn fetch(url: &Url) -> Result<Self, ExtensionError> {
        let resp = reqwest::blocking::get(url.clone())
            .map_err(|e| ExtensionError::CompatibilityMatrixFetchError(url.to_string(), e))?;
        resp.json()
            .map_err(ExtensionError::MalformedCompatibilityMatrix)
    }

    pub fn find_extension(
        &self,
        extension_name: &str,
        dfx_version: &Version,
    ) -> Result<ExtensionManifest, ExtensionError> {
        let extension_version = self
            .compatibility
            .find_latest_compatible_extension_version(extension_name, dfx_version)?;
        let mut manifest = self
            .extensions
            .get(extension_name)
            .ok_or_else(|| {
                ExtensionError::ExtensionNameNotFoundInManifest(extension_name.to_string())
            })?
            .clone()
            .get(&extension_version)
            .ok_or_else(|| {
                ExtensionError::MalformedManifestExtensionVersionNotFound(
                    extension_name.to_string(),
                    extension_version.clone(),
                )
            })?
            .clone();
        manifest.version.replace(extension_version.to_string());
        manifest.name.replace(extension_name.to_string());
        Ok(manifest)
    }
}
