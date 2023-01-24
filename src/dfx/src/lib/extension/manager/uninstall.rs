use crate::lib::error::{DfxError, DfxResult, ExtensionError};

use super::ExtensionsManager;

impl ExtensionsManager {
    pub fn uninstall_extension(&self, extension_name: &str) -> DfxResult<()> {
        let path = self.get_extension_directory(extension_name);
        if let Err(e) = std::fs::remove_dir_all(path) {
            return Err(DfxError::new(
                ExtensionError::InsufficientPermissionsToDeleteExtensionDirectory(e),
            ));
        }
        Ok(())
    }
}
