use std::ffi::OsString;
use std::path::PathBuf;

use semver::{Prerelease, Version};

use crate::config::cache::{get_bin_cache, install_version, Cache, DiskBasedCache};
use crate::lib::error::{CacheError, DfxError, DfxResult};

mod install;
pub mod manifest;

pub trait ExtensionsManager {
    fn init_extensions_directory(&self) -> Result<(), std::io::Error>;
    fn get_extensions_directory(&self) -> DfxResult<PathBuf>;
    fn get_extension_directory(&self, extension_name: &str) -> DfxResult<PathBuf>;
    fn get_extension_binary(&self, extension_name: &str) -> DfxResult<std::process::Command>;
    fn install_extension(&self, dfx_version: &Version, extension_name: &str) -> DfxResult<()>;
    fn uninstall_extension(&self, extension_name: &str) -> DfxResult<()>;
    fn list_installed_extensions(&self);
    fn run_extension(&self, extension_name: OsString, params: Vec<OsString>) -> DfxResult<()>;
    fn display_help_for_extension(&self);
    fn list_compatible_extensions(&self) -> &'static str;
    fn upgrade_extension(&self) -> &'static str;
}

impl ExtensionsManager for DiskBasedCache {
    fn init_extensions_directory(&self) -> Result<(), std::io::Error> {
        let path = self.get_extensions_directory().unwrap();
        std::fs::create_dir_all(&path)
    }

    fn get_extensions_directory(&self) -> DfxResult<PathBuf> {
        let version = &self.version_str();
        install_version(version, false)?;
        Ok(get_bin_cache(version)?.join("extensions"))
    }

    fn get_extension_directory(&self, extension_name: &str) -> DfxResult<PathBuf> {
        self.get_extensions_directory()
            .map(|dir| dir.join(extension_name))
    }

    fn get_extension_binary(&self, extension_name: &str) -> DfxResult<std::process::Command> {
        if let Ok(dir) = self.get_extension_directory(extension_name) {
            let bin = dir.join(extension_name);
            if bin.exists() && bin.is_file() {
                Ok(std::process::Command::new(bin))
            } else {
                Err(DfxError::new(CacheError::CreateCacheDirectoryFailed(
                    PathBuf::from(extension_name.to_string()),
                )))
                // Err(DfxError::ExtensionError(format!(
                //     "Extension {} is not installed",
                //     extension_name
                // )))
            }
        } else {
            Err(DfxError::new(CacheError::CreateCacheDirectoryFailed(
                PathBuf::from(extension_name.to_string()),
            )))
        }
    }

    fn install_extension(&self, dfx_version: &Version, extension_name: &str) -> DfxResult<()> {
        let mut v = dfx_version.clone();
        // Remove the prerelease tag, if any. This is because prerelease tags
        // should not be allowed in extension manifests, and semver crate
        // won't match a semver with a prerelease tag against a semver without.
        v.pre = Prerelease::EMPTY;

        let p = self.get_extensions_directory().unwrap();

        install::install_extension(&p, v, extension_name)
    }

    fn uninstall_extension(&self, extension_name: &str) -> DfxResult<()> {
        if let Ok(path) = self.get_extension_directory(extension_name) {
            std::fs::remove_dir_all(path).unwrap();
            return Ok(());
        }
        Err(DfxError::new(CacheError::CreateCacheDirectoryFailed(
            PathBuf::from(extension_name.to_string()),
        )))
    }

    fn list_installed_extensions(&self) {
        let x = self.get_extensions_directory().unwrap().read_dir().unwrap();

        let mut counter = 0;
        let mut names = Vec::new();

        for entry in x {
            counter += 1;
            let entry = entry.unwrap();
            let path = entry.path();
            let name = path.file_name().unwrap().to_str().unwrap();
            names.push(format!("  {}", name));
        }
        if counter == 0 {
            println!("No extensions installed");
        } else {
            println!("Installed extensions:");
            for name in names {
                println!("{}", name);
            }
        }
    }

    fn run_extension(&self, extension_name: OsString, params: Vec<OsString>) -> DfxResult<()> {
        if let Ok(mut extension_binary) =
            self.get_extension_binary(extension_name.to_str().unwrap())
        {
            return extension_binary
                .args(&params)
                .spawn()
                .expect("failed to execute process")
                .wait()
                .expect("failed to wait on child")
                .code()
                .map_or(Ok(()), |code| {
                    Err(anyhow::anyhow!("Extension exited with code {}", code))
                });
        } else {
            Err(anyhow::anyhow!(
                "extension {:?} does cannot be found",
                extension_name
            ))
        }
    }

    fn display_help_for_extension(&self) {
        todo!()
    }

    fn list_compatible_extensions(&self) -> &'static str {
        todo!()
    }

    fn upgrade_extension(&self) -> &'static str {
        todo!()
    }
}
