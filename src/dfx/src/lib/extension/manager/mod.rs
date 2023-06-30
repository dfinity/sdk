use crate::{config::dfx_version, lib::error::ExtensionError};
use clap::CommandFactory;
use dfx_core::config::cache::{get_bin_cache, get_cache_root};

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

impl CommandFactory for ExtensionManager {
    fn command() -> clap::Command {
        ExtensionManager::new(dfx_version(), false).map_or_else(
            |_| clap::Command::new("empty"),
            |mgr| {
                clap::Command::new("installed-extensions").subcommands(
                    mgr.list_installed_extensions()
                        .unwrap_or_default()
                        .into_iter()
                        .map(|v| v.into_clap_command(&mgr)),
                )
            },
        )
    }

    fn command_for_update() -> clap::Command {
        Self::command()
    }
}

impl ExtensionManager {
    pub fn new(version: &Version, ensure_dir_exists: bool) -> Result<Self, ExtensionError> {
        let versioned_cache_dir = get_bin_cache(version.to_string().as_str()).map_err(|e| {
            ExtensionError::FindCacheDirectoryFailed(
                get_cache_root()
                    .unwrap_or_default()
                    .join("versions")
                    .join(version.to_string()),
                e,
            )
        })?;
        let dir = versioned_cache_dir.join("extensions");
        if ensure_dir_exists {
            dfx_core::fs::composite::ensure_dir_exists(&dir)
                .map_err(ExtensionError::EnsureExtensionDirExistsFailed)?;
        }

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

    pub fn is_extension_installed(&self, extension_name: &str) -> bool {
        self.get_extension_directory(extension_name).exists()
    }
}
