use std::path::{Path, PathBuf};
use serde_json::Value;

pub const CONFIG_FILE_NAME: &str = "dfinity.json";

pub struct Config {
    path: PathBuf,
    value: Value,
}

impl Config {
    pub fn resolve_config_path(working_dir: &Path) -> Result<PathBuf, std::io::Error> {
        fn recurse(mut curr: PathBuf) -> Result<PathBuf, std::io::Error> {
            curr.push(CONFIG_FILE_NAME);

            if curr.is_file() {
                Ok(curr)
            } else {
                curr.pop();
                match curr.pop() {
                    false => Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Config not found.")),
                    true => recurse(curr),
                }
            }
        }

        recurse(PathBuf::from(working_dir))
    }

    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }
    pub fn get_value(&self) -> &Value {
        &self.value
    }

    pub fn load_from(working_dir: &PathBuf) -> std::io::Result<Config> {
        let path = Config::resolve_config_path(working_dir)?;
        let content = std::fs::read(&path)?;
        let value = serde_json::from_slice(&content)?;
        Ok(Config{path, value})
    }
}
