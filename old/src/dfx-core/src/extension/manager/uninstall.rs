use super::ExtensionManager;
use crate::error::extension::UninstallExtensionError;

impl ExtensionManager {
    pub fn uninstall_extension(&self, extension_name: &str) -> Result<(), UninstallExtensionError> {
        let path = self.get_extension_directory(extension_name);
        crate::fs::remove_dir_all(&path)?;
        Ok(())
    }
}
