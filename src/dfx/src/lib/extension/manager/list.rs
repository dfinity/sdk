use super::ExtensionManager;
use crate::lib::{error::ExtensionError, extension::Extension};

impl ExtensionManager {
    pub fn list_installed_extensions(&self) -> Result<Vec<Extension>, ExtensionError> {
        let dir_content = dfx_core::fs::read_dir(&self.dir)
            .map_err(ExtensionError::ExtensionsDirectoryIsNotReadable)?;

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

    pub fn installed_extensions_as_clap_commands(
        &self,
    ) -> Result<Vec<clap::Command>, ExtensionError> {
        let mut extensions = vec![];
        for ext in self.list_installed_extensions()? {
            extensions.push(ext.into_clap_command(self)?);
        }
        Ok(extensions)
    }
}
