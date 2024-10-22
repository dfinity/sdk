use super::ExtensionManager;
use crate::error::extension::{ListAvailableExtensionsError, ListInstalledExtensionsError};
use crate::extension::catalog::ExtensionCatalog;
use crate::extension::installed::InstalledExtensionList;
use crate::extension::ExtensionName;
use std::vec;
use url::Url;

pub type AvailableExtensionList = Vec<ExtensionName>;

impl ExtensionManager {
    pub fn list_installed_extensions(
        &self,
    ) -> Result<InstalledExtensionList, ListInstalledExtensionsError> {
        if !self.dir.exists() {
            return Ok(vec![]);
        }
        let dir_content = crate::fs::read_dir(&self.dir)?;

        let extensions = dir_content
            .filter_map(|v| {
                let dir_entry = v.ok()?;
                if dir_entry.file_type().map_or(false, |e| e.is_dir())
                    && !dir_entry.file_name().to_str()?.starts_with(".tmp")
                {
                    let name = dir_entry.file_name().to_string_lossy().to_string();
                    Some(name)
                } else {
                    None
                }
            })
            .collect();
        Ok(extensions)
    }

    pub async fn list_available_extensions(
        &self,
        catalog_url: Option<&Url>,
    ) -> Result<AvailableExtensionList, ListAvailableExtensionsError> {
        let catalog = ExtensionCatalog::fetch(catalog_url)
            .await
            .map_err(ListAvailableExtensionsError::FetchCatalog)?;
        let extensions: Vec<String> = catalog.0.into_keys().collect();

        Ok(extensions)
    }
}
