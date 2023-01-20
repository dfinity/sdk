use crate::config::cache::get_bin_cache;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult, ExtensionError};
use semver::Version;
use std::path::PathBuf;

mod execute;
mod install;
mod list;
mod uninstall;

pub struct ExtensionsManager {
    pub dir: PathBuf,
    pub dfx_version: Version,
}

impl ExtensionsManager {
    pub fn new(env: &dyn Environment) -> DfxResult<Self> {
        if let Ok(x) = get_bin_cache(env.get_version().to_string().as_str()) {
            let dir = x.join("extensions");
            std::fs::create_dir_all(&dir)?;
            Ok(Self {
                dir,
                dfx_version: env.get_version().clone(),
            })
        } else {
            Err(DfxError::new(ExtensionError::ExtensionError(
                "Unable to get bin cache".to_string(),
            )))
        }
    }

    pub fn get_extension_directory(&self, extension_name: &str) -> DfxResult<PathBuf> {
        let dir = self.dir.join(extension_name);
        if !dir.exists() {
            return Err(DfxError::new(ExtensionError::ExtensionError(
                extension_name.to_string(),
            )));
        }
        Ok(dir)
    }

    pub fn get_extension_binary(&self, extension_name: &str) -> DfxResult<std::process::Command> {
        let dir = self.get_extension_directory(extension_name)?;
        let bin = dir.join(extension_name);
        if bin.exists() && bin.is_file() {
            Ok(std::process::Command::new(bin))
        } else {
            Err(DfxError::new(ExtensionError::ExtensionError(format!(
                "Extension {} is not installed",
                extension_name
            ))))
        }
    }
}
