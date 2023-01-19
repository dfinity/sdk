use super::ExtensionsManager;
use crate::lib::{error::DfxResult, extension::Extension};

// possible errors:
// - extensions directory does not exist
// - extensions directory is not a directory
// - extensions directory is not readable

impl ExtensionsManager {
    pub fn list_installed_extensions(&self) -> DfxResult<Vec<Extension>> {
        Ok(self
            .dir
            .read_dir()?
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
