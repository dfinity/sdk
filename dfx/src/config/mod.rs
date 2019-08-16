#![allow(dead_code)]
use crate::commands::CliResult;
use serde_json::{Value, Map};
use std::path::{Path, PathBuf};

pub const CONFIG_FILE_NAME: &str = "dfinity.json";

pub struct Config {
    path: PathBuf,
    config: Map<String, Value>,
}

impl Config {
    pub fn resolve_config_path(working_dir: &Path) -> Result<PathBuf, std::io::Error> {
        fn recurse(mut curr: PathBuf) -> Result<PathBuf, std::io::Error> {
            curr.push(CONFIG_FILE_NAME);

            if curr.is_file() {
                Ok(curr)
            } else {
                curr.pop();  // Remove the filename.
                if curr.pop() {
                    recurse(curr)
                } else {
                    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Config not found."))
                }
            }
        }

        recurse(PathBuf::from(working_dir))
    }

    pub fn load_from(working_dir: &PathBuf) -> std::io::Result<Config> {
        let path = Config::resolve_config_path(working_dir)?;
        let content = std::fs::read(&path)?;
        let config = serde_json::from_slice(&content)?;
        Ok(Config{path, config})
    }

    pub fn from_current_dir() -> std::io::Result<Config> {
        Config::load_from(&std::env::current_dir()?)
    }

    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }
    pub fn get_value(&self) -> &Map<String, Value> { &self.config }
    pub fn get_mut_value(&mut self) -> &mut Map<String, Value> { &mut self.config }

    pub fn save(&self) -> CliResult {
        std::fs::write(&self.path, serde_json::to_string_pretty(&self.config).unwrap())?;
        Ok(())
    }
}
