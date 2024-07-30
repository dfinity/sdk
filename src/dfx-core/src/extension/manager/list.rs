use super::ExtensionManager;
use crate::error::extension::{ConvertExtensionIntoClapCommandError, ListInstalledExtensionsError};
use crate::extension::Extension;

impl ExtensionManager {
    pub fn list_installed_extensions(
        &self,
    ) -> Result<Vec<Extension>, ListInstalledExtensionsError> {
        if !&self.dir.exists() {
            return Ok(vec![]);
        }
        let dir_content = crate::fs::read_dir(&self.dir)?;

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
    ) -> Result<Vec<clap::Command>, ConvertExtensionIntoClapCommandError> {
        let mut extensions = vec![];
        for ext in self.list_installed_extensions()? {
            extensions.push(ext.into_clap_command(self)?);
        }
        Ok(extensions)
    }
}
