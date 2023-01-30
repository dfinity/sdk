use crate::lib::error::ExtensionError;

use super::ExtensionManager;

impl ExtensionManager {
    pub fn uninstall_extension(&self, extension_name: &str) -> Result<(), ExtensionError> {
        let path = self.get_extension_directory(extension_name);
        if let Err(e) = std::fs::remove_dir_all(path) {
            return Err(ExtensionError::InsufficientPermissionsToDeleteExtensionDirectory(e));
        }
        Ok(())
    }
}
