use crate::commands::CliResult;
use serde::{Deserialize, Serialize};
use serde_json::{Value, Map};
use std::path::{Path, PathBuf};

pub const CONFIG_FILE_NAME: &str = "dfinity.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigServerInterface {
    address: Option<String>,
    port: Option<u16>,
    nodes: Option<u64>,
}

impl ConfigServerInterface {
    pub fn get_address(&self, default: String) -> String {
        match &self.address {
            Some(v) => v.clone(),
            _ => default,
        }
    }
    pub fn get_nodes(&self, default: u64) -> u64 {
        match self.nodes {
            Some(v) => v,
            _ => default,
        }
    }
    pub fn get_port(&self, default: u16) -> u16 {
        match self.port {
            Some(v) => v,
            _ => default,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigInterface {
    server: Option<ConfigServerInterface>,
}

static EMPTY_CONFIG_SERVER: ConfigServerInterface = ConfigServerInterface{
    address: None,
    port: None,
    nodes: None,
};
impl ConfigInterface {
    pub fn get_server(&self) -> &ConfigServerInterface {
        match &self.server {
            Some(v) => &v,
            _ => &EMPTY_CONFIG_SERVER,
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
