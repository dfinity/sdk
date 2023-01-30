use crate::lib::error::ExtensionError;

use super::ExtensionManager;

impl ExtensionManager {
    pub fn uninstall_extension(&self, extension_name: &str) -> Result<(), ExtensionError> {
        let path = self.get_extension_directory(extension_name);
        dfx_core::fs::remove_dir_all(&path)
            .map_err(ExtensionError::InsufficientPermissionsToDeleteExtensionDirectory)?;
        Ok(())
    }
}
