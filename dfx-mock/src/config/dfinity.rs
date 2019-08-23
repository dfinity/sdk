use crate::commands::CliResult;
use serde::{Deserialize, Serialize};
use serde_json::{Value, Map};
use std::path::{Path, PathBuf};
use crate::config::DFX_VERSION;

pub const CONFIG_FILE_NAME: &str = "dfinity.json";


#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigDefaults {
    pub address: Option<String>,
    pub nodes: Option<u64>,
    pub port: Option<u16>,
}

pub const EMPTY_CONFIG_SERVER: ConfigDefaults = ConfigDefaults{
    address: None,
    port: None,
    nodes: None,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigInterface {
    pub dfx: Option<String>,
    //    pub canisters: Option<Map<String, ConfigCanisters>>
    pub defaults: Option<ConfigDefaults>,
}

impl ConfigInterface {
    pub fn get_defaults(&self) -> &ConfigDefaults {
        match &self.defaults {
            Some(v) => &v,
            _ => &EMPTY_CONFIG_SERVER,
        }
    }
    pub fn get_dfx_version(&self) -> String {
        match &self.dfx {
            Some(v) => v.to_owned(),
            _ => DFX_VERSION.to_owned(),
        }
    }
}

pub struct Config {
    path: PathBuf,
    json_object: Map<String, Value>,
    config: ConfigInterface,
}

#[allow(dead_code)]
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
        let json_object = serde_json::from_slice(&content)?;
        Ok(Config{path, json_object, config})
    }

    pub fn from_current_dir() -> std::io::Result<Config> {
        Config::load_from(&std::env::current_dir()?)
    }

    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }
    pub fn get_value(&self) -> &Map<String, Value> { &self.json_object }
    pub fn get_mut_value(&mut self) -> &mut Map<String, Value> { &mut self.json_object }
    pub fn get_config(&self) -> &ConfigInterface { &self.config }

    pub fn save(&self) -> CliResult {
        std::fs::write(&self.path, serde_json::to_string_pretty(&self.json_object).unwrap())?;
        Ok(())
    }
}
