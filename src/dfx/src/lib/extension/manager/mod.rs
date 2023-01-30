use crate::config::cache::{get_bin_cache, get_cache_root};
use crate::lib::environment::Environment;
use crate::lib::error::ExtensionError;
use crate::lib::extension::manifest::MANIFEST_FILE_NAME;

use semver::Version;
use std::path::PathBuf;
use dfx_core::fs::composite::ensure_dir_exists;

use super::manifest::ExtensionManifest;
use super::Extension;

mod execute;
mod install;
mod list;
mod uninstall;

pub struct ExtensionManager {
    pub dir: PathBuf,
    pub dfx_version: Version,
}

impl ExtensionManager {
    pub fn new(env: &dyn Environment) -> Result<Self, ExtensionError> {
        let versioned_cache_dir = get_bin_cache(env.get_version().to_string().as_str()).map_err(|e| {
            ExtensionError::FindCacheDirectoryFailed(
                get_cache_root().unwrap_or_default().join("versions"), e
            )
        })?;
        let dir = versioned_cache_dir.join("extensions");
        ensure_dir_exists(&dir).map_err(|e| {
            ExtensionError::EnsureExtensionDirExistsFailed(e)
        })?;
        
        Ok(Self {
            dir,
            dfx_version: env.get_version().clone(),
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
            Err(ExtensionError::ExtensionBinaryDoesNotExist(
                extension_name.to_string(),
            ))
        } else if !bin.is_file() {
            Err(ExtensionError::ExtensionBinaryIsNotAFile(
                extension_name.to_string(),
            ))
        } else {
            Ok(std::process::Command::new(bin))
        }
    }

    pub fn load_manifest(&self, ext: Extension) -> Result<ExtensionManifest, ExtensionError> {
        let manifest_path = self
            .get_extension_directory(&ext.name)
            .join(MANIFEST_FILE_NAME);
        let manifest_file = std::fs::File::open(&manifest_path)
            .map_err(|e| ExtensionError::ExtensionManifestDoesNotExist(manifest_path.clone(), e))?;

        let manifest_reader = std::io::BufReader::new(manifest_file);

        serde_json::from_reader(manifest_reader)
            .map_err(|e| ExtensionError::ExtensionManifestIsNotValidJson(manifest_path.clone(), e))
    }
}
