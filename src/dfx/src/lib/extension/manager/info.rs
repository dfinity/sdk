use crate::lib::{extension::manifest::ExtensionManifest, error::DfxResult};
use super::ExtensionsManager;
use std::fs::File;

impl ExtensionsManager {
    pub fn get_extension_metadata(&self, extension_name: &str) -> DfxResult<ExtensionManifest> {
        let spec_path = self
            .dir
            .join(extension_name)
            .join("manifest.json");
        let file = File::open(spec_path).unwrap();
        let reader = std::io::BufReader::new(file);
        let manifest: ExtensionManifest = serde_json::from_reader(reader)?;
        Ok(manifest)
    }
}
