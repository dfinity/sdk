use crate::config::cache::get_cache_path_for_version;
use crate::error::extension::{GetExtensionBinaryError, NewExtensionManagerError};
use semver::Version;
use std::path::PathBuf;
use url::Url;

mod execute;
mod install;
mod list;
mod uninstall;

pub struct ExtensionManager {
    pub dir: PathBuf,
    pub dfx_version: Version,
}

impl ExtensionManager {
    pub fn new(version: &Version) -> Result<Self, NewExtensionManagerError> {
        let extensions_dir = get_cache_path_for_version(&version.to_string())
            .map_err(NewExtensionManagerError::FindCacheDirectoryFailed)?
            .join("extensions");

        Ok(Self {
            dir: extensions_dir,
            dfx_version: version.clone(),
        })
    }

    pub fn get_extension_directory(&self, extension_name: &str) -> PathBuf {
        self.dir.join(extension_name)
    }

    pub fn get_extension_binary(
        &self,
        extension_name: &str,
    ) -> Result<std::process::Command, GetExtensionBinaryError> {
        let dir = self.get_extension_directory(extension_name);
        if !dir.exists() {
            return Err(GetExtensionBinaryError::ExtensionNotInstalled(
                extension_name.to_string(),
            ));
        }
        let bin = dir.join(extension_name);
        if !bin.exists() {
            Err(GetExtensionBinaryError::ExtensionBinaryDoesNotExist(bin))
        } else if !bin.is_file() {
            Err(GetExtensionBinaryError::ExtensionBinaryIsNotAFile(bin))
        } else {
            Ok(std::process::Command::new(bin))
        }
    }

    pub fn is_extension_installed(&self, extension_name: &str) -> bool {
        self.get_extension_directory(extension_name).exists()
    }

    pub fn get_extension_base_url(&self, name: &str) -> Result<Url, url::ParseError> {
        let s = format!("https://raw.githubusercontent.com/dfinity/dfx-extensions/main/extensions/{name}/");
        Url::parse(&s)
    }
}
