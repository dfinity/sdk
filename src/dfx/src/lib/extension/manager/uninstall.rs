use std::path::PathBuf;

use crate::lib::error::{CacheError, DfxError, DfxResult};

use super::ExtensionsManager;

impl ExtensionsManager {
    pub fn uninstall_extension(&self, extension_name: &str) -> DfxResult<()> {
        if let Ok(path) = self.get_extension_directory(extension_name) {
            std::fs::remove_dir_all(path).unwrap();
            return Ok(());
        }
        Err(DfxError::new(CacheError::CreateCacheDirectoryFailed(
            PathBuf::from(extension_name.to_string()),
        )))
    }
}
