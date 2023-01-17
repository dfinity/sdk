use super::ExtensionsManager;
use crate::lib::extension::Extension;

impl ExtensionsManager {
    pub fn list_installed_extensions(&self) -> Vec<Extension> {
        self.dir
            .read_dir()
            .unwrap()
            .filter_map(|v| {
                if v.is_ok()
                    && v.as_ref().unwrap().file_type().unwrap().is_dir()
                    && !v.as_ref()
                        .unwrap()
                        .file_name()
                        .to_str()
                        .unwrap()
                        .starts_with(".tmp")
                {
                    Some(Extension::from(v.unwrap()))
                } else {
                    None
                }
            })
            .collect()
    }
}
