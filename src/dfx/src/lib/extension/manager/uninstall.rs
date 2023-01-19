use crate::lib::error::DfxResult;

use super::ExtensionsManager;

// possible errors:
// - insufficient permissions to delete the directory
// - directory does not exist

impl ExtensionsManager {
    pub fn uninstall_extension(&self, extension_name: &str) -> DfxResult<()> {
        let path = self.get_extension_directory(extension_name)?;
        std::fs::remove_dir_all(path)?;
        Ok(())
    }
}
