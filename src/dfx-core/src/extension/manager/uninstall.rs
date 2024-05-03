use super::ExtensionManager;
use crate::error::extension::ExtensionError;

impl ExtensionManager {
    pub fn uninstall_extension(&self, extension_name: &str) -> Result<(), ExtensionError> {
        let path = self.get_extension_directory(extension_name);
        crate::fs::remove_dir_all(&path)
            .map_err(ExtensionError::InsufficientPermissionsToDeleteExtensionDirectory)?;
        Ok(())
    }
}
