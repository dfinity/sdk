#![allow(dead_code)]

use crate::lib::error::DfxResult;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};

pub const CONFIG_FILE_NAME: &str = "dfinity.json";

const EMPTY_CONFIG_DEFAULTS: ConfigDefaults = ConfigDefaults {
    build: None,
    start: None,
};
const EMPTY_CONFIG_DEFAULTS_START: ConfigDefaultsStart = ConfigDefaultsStart {
    address: None,
    port: None,
    nodes: None,
};
const EMPTY_CONFIG_DEFAULTS_BUILD: ConfigDefaultsBuild = ConfigDefaultsBuild { output: None };

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigCanistersCanister {
    pub main: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigDefaultsStart {
    pub address: Option<String>,
    pub nodes: Option<u64>,
    pub port: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigDefaultsBuild {
    pub output: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigDefaults {
    pub build: Option<ConfigDefaultsBuild>,
    pub start: Option<ConfigDefaultsStart>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigInterface {
    pub version: Option<u32>,
    pub dfx: Option<String>,
    pub canisters: Option<Map<String, Value>>,
    pub defaults: Option<ConfigDefaults>,
}

impl ConfigCanistersCanister {
    pub fn get_main(&self, default: &str) -> String {
        self.main.to_owned().unwrap_or_else(|| default.to_string())
    }
}

impl ConfigDefaultsStart {
    pub fn get_address(&self, default: &str) -> String {
        self.address
            .to_owned()
            .unwrap_or_else(|| default.to_string())
    }
    pub fn get_nodes(&self, default: u64) -> u64 {
        self.nodes.unwrap_or(default)
    }
    pub fn get_port(&self, default: u16) -> u16 {
        self.port.unwrap_or(default)
    }
}

impl ConfigDefaultsBuild {
    pub fn get_output(&self, default: &str) -> String {
        self.output
            .to_owned()
            .unwrap_or_else(|| default.to_string())
    }
}

impl ConfigDefaults {
    pub fn get_build(&self) -> &ConfigDefaultsBuild {
        match &self.build {
            Some(x) => &x,
            None => &EMPTY_CONFIG_DEFAULTS_BUILD,
        }
    }
    pub fn get_start(&self) -> &ConfigDefaultsStart {
        match &self.start {
            Some(x) => &x,
            None => &EMPTY_CONFIG_DEFAULTS_START,
        }
    }
}

impl ConfigInterface {
    pub fn get_defaults(&self) -> &ConfigDefaults {
        match &self.defaults {
            Some(v) => &v,
            _ => &EMPTY_CONFIG_DEFAULTS,
        }
    }
    pub fn get_version(&self) -> u32 {
        self.version.unwrap_or(1)
    }
    pub fn get_dfx(&self) -> Option<String> {
        self.dfx.to_owned()
    }
}

pub struct Config {
    path: PathBuf,
    json: Value,
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
                curr.pop(); // Remove the filename.
                if curr.pop() {
                    recurse(curr)
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "Config not found.",
                    ))
                }
            }
        }

        recurse(PathBuf::from(working_dir))
    }

    pub fn load_from(working_dir: &PathBuf) -> std::io::Result<Config> {
        let path = Config::resolve_config_path(working_dir)?;
        let content = std::fs::read(&path)?;
        let config = serde_json::from_slice(&content)?;
        let json = serde_json::from_slice(&content)?;
        Ok(Config { path, json, config })
    }

    pub fn from_current_dir() -> std::io::Result<Config> {
        Config::load_from(&std::env::current_dir()?)
    }

    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }
    pub fn get_json(&self) -> &Value {
        &self.json
    }
    pub fn get_mut_json(&mut self) -> &mut Value {
        &mut self.json
    }
    pub fn get_config(&self) -> &ConfigInterface {
        &self.config
    }

    pub fn save(&self) -> DfxResult {
        std::fs::write(
            &self.path,
            serde_json::to_string_pretty(&self.json).unwrap(),
        )?;
        Ok(())
    }
}
