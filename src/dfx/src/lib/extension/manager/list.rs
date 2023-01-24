use super::ExtensionsManager;
use crate::lib::{
    error::{DfxError, DfxResult, ExtensionError},
    extension::Extension,
};

impl ExtensionsManager {
    pub fn list_installed_extensions(&self) -> DfxResult<Vec<Extension>> {
        let Ok(dir_content) = self.dir.read_dir() else {
            return Err(DfxError::new(ExtensionError::ExtensionsDirectoryIsNotReadable));
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
