use crate::config::cache::{get_bin_cache, get_cache_root};
use crate::error::extension::ExtensionError;
use crate::extension::{
    manifest::{ExtensionManifest, MANIFEST_FILE_NAME},
    Extension,
};

use crate::fs::composite::ensure_dir_exists;
use crate::json::load_json_file;
use semver::Version;
use std::path::PathBuf;

mod execute;
mod install;
mod list;
mod uninstall;

pub struct ExtensionManager {
    pub dir: PathBuf,
    pub dfx_version: Version,
}

impl ExtensionManager {
    pub fn new(version: &Version) -> Result<Self, ExtensionError> {
        let versioned_cache_dir = get_bin_cache(version.to_string().as_str()).map_err(|e| {
            ExtensionError::FindCacheDirectoryFailed(
                get_cache_root().unwrap_or_default().join("versions"),
                e,
            )
        })?;
        let dir = versioned_cache_dir.join("extensions");
        ensure_dir_exists(&dir).map_err(ExtensionError::EnsureExtensionDirExistsFailed)?;

        Ok(Self {
            dir,
            dfx_version: version.clone(),
        })
    }

    pub fn get_extension_directory(&self, extension_name: &str) -> PathBuf {
        self.dir.join(extension_name)
    }

    pub fn get_extension_binary(
        &self,
        extension_name: &str,
    ) -> Result<std::process::Command, ExtensionError> {
        let dir = self.get_extension_directory(extension_name);
        if !dir.exists() {
            return Err(ExtensionError::ExtensionNotInstalled(
                extension_name.to_string(),
            ));
        }
        let bin = dir.join(extension_name);
        if !bin.exists() {
            Err(ExtensionError::ExtensionBinaryDoesNotExist(bin))
        } else if !bin.is_file() {
            Err(ExtensionError::ExtensionBinaryIsNotAFile(bin))
        } else {
            Ok(std::process::Command::new(bin))
        }
    }

    #[allow(dead_code)]
    pub fn load_manifest(&self, ext: Extension) -> Result<ExtensionManifest, ExtensionError> {
        let manifest_path = self
            .get_extension_directory(&ext.name)
            .join(MANIFEST_FILE_NAME);
        load_json_file(&manifest_path).map_err(ExtensionError::ExtensionManifestIsNotValidJson)
    }
}
