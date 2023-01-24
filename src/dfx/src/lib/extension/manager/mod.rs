use crate::config::cache::{get_bin_cache, get_cache_root};
use crate::lib::environment::Environment;
use crate::lib::error::{CacheError, DfxError, DfxResult, ExtensionError};
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
    pub fn new(env: &dyn Environment) -> DfxResult<Self> {
        let Ok(x) = get_bin_cache(env.get_version().to_string().as_str())
         else {
            return Err(DfxError::new(CacheError::FindCacheDirectoryFailed(
                get_cache_root()?.join("versions"),
            )))
        };
        let dir = x.join("extensions");
        if !dir.exists() {
            if let Err(_e) = std::fs::create_dir_all(&dir) {
                return Err(DfxError::new(
                    ExtensionError::CreateExtensionDirectoryFailed(dir),
                ));
            }
        }
        if !dir.is_dir() {
            return Err(DfxError::new(
                ExtensionError::ExtensionsDirectoryIsNotADirectory,
            ));
        }
        Ok(Self {
            dir,
            dfx_version: env.get_version().clone(),
        })
    }

    pub fn get_extension_directory(&self, extension_name: &str) -> PathBuf {
        self.dir.join(extension_name)
    }

    pub fn get_extension_binary(&self, extension_name: &str) -> DfxResult<std::process::Command> {
        let dir = self.get_extension_directory(extension_name);
        if !dir.exists() {
            return Err(DfxError::new(ExtensionError::ExtensionNotInstalled(
                extension_name.to_string(),
            )));
        }
        let bin = dir.join(extension_name);
        if !bin.exists() {
            Err(DfxError::new(ExtensionError::ExtensionBinaryDoesNotExist(
                extension_name.to_string(),
            )))
        } else if !bin.is_file() {
            Err(DfxError::new(ExtensionError::ExtensionBinaryIsNotAFile(
                extension_name.to_string(),
            )))
        } else {
            Ok(std::process::Command::new(bin))
        }
    }
}
