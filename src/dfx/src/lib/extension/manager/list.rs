use super::ExtensionManager;
use crate::lib::{error::ExtensionError, extension::Extension};

impl ExtensionManager {
    pub fn list_installed_extensions(&self) -> Result<Vec<Extension>, ExtensionError> {
        let Ok(dir_content) = self.dir.read_dir() else {
            return Err(ExtensionError::ExtensionsDirectoryIsNotReadable);
        };

        Ok(dir_content
            .filter_map(|v| {
                let dir_entry = v.ok()?;
                if dir_entry.file_type().map_or(false, |e| e.is_dir())
                    && !dir_entry.file_name().to_str()?.starts_with(".tmp")
                {
                    Some(Extension::from(dir_entry))
                } else {
                    None
                }
            })
            .collect())
    }
}
