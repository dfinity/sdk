#![allow(dead_code)]

use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use clap::Clap;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::default::Default;
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};
use url::Url;

pub const CONFIG_FILE_NAME: &str = "dfx.json";

const EMPTY_CONFIG_DEFAULTS: ConfigDefaults = ConfigDefaults {
    bootstrap: None,
    build: None,
    replica: None,
    start: None,
};

lazy_static! {
    static ref EMPTY_CONFIG_DEFAULTS_BOOTSTRAP: ConfigDefaultsBootstrap = ConfigDefaultsBootstrap {
        ip: None,
        port: None,
        providers: vec![],
        root: None,
        timeout: None,
    };
}

const EMPTY_CONFIG_DEFAULTS_BUILD: ConfigDefaultsBuild = ConfigDefaultsBuild { output: None };

const EMPTY_CONFIG_DEFAULTS_REPLICA: ConfigDefaultsReplica = ConfigDefaultsReplica {
    message_gas_limit: None,
    port: None,
    round_gas_limit: None,
};

const EMPTY_CONFIG_DEFAULTS_START: ConfigDefaultsStart = ConfigDefaultsStart {
    address: None,
    port: None,
    nodes: None,
    serve_root: None,
};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ConfigCanistersCanister {
    pub main: Option<String>,
    pub frontend: Option<Value>,
}

#[derive(Clap, Clone, Debug, Default, Serialize, Deserialize)]
pub struct ConfigDefaultsBootstrap {
    #[clap(help = UserMessage::BootstrapIP.to_str(), long = "ip", takes_value = true)]
    pub ip: Option<IpAddr>,
    #[clap(help = UserMessage::BootstrapPort.to_str(), long = "port", takes_value = true)]
    pub port: Option<u16>,
    #[clap(help = UserMessage::BootstrapRoot.to_str(), long = "root", takes_value = true)]
    pub root: Option<PathBuf>,
    #[clap(help = UserMessage::BootstrapProviders.to_str(), long = "providers", multiple = true, takes_value = true)]
    pub providers: Vec<Url>,
    #[clap(help = UserMessage::BootstrapTimeout.to_str(), long = "timeout", takes_value = true)]
    pub timeout: Option<u64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ConfigDefaultsBuild {
    pub output: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ConfigDefaultsReplica {
    pub message_gas_limit: Option<u64>,
    pub port: Option<u16>,
    pub round_gas_limit: Option<u64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ConfigDefaultsStart {
    pub address: Option<String>,
    pub nodes: Option<u64>,
    pub port: Option<u16>,
    pub serve_root: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Profile {
    // debug is for development only
    Debug,
    // release is for production
    Release,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigDefaults {
    pub bootstrap: Option<ConfigDefaultsBootstrap>,
    pub build: Option<ConfigDefaultsBuild>,
    pub replica: Option<ConfigDefaultsReplica>,
    pub start: Option<ConfigDefaultsStart>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigInterface {
    pub profile: Option<Profile>,
    pub version: Option<u32>,
    pub dfx: Option<String>,
    pub canisters: Option<BTreeMap<String, ConfigCanistersCanister>>,
    pub defaults: Option<ConfigDefaults>,
}

impl ConfigCanistersCanister {
    pub fn get_main(&self, default: &str) -> String {
        self.main.to_owned().unwrap_or_else(|| default.to_string())
    }
}

fn to_socket_addr(s: &str) -> DfxResult<SocketAddr> {
    match s.to_socket_addrs() {
        Ok(mut a) => match a.next() {
            Some(res) => Ok(res),
            None => Err(DfxError::from(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Empty iterator",
            ))),
        },
        Err(err) => Err(DfxError::from(err)),
    }
}

impl ConfigDefaultsStart {
    pub fn get_address(&self, default: &str) -> String {
        self.address
            .to_owned()
            .unwrap_or_else(|| default.to_string())
    }
    pub fn get_binding_socket_addr(&self, default: &str) -> DfxResult<SocketAddr> {
        to_socket_addr(default).and_then(|default_addr| {
            let addr = self.get_address(default_addr.ip().to_string().as_str());
            let port = self.get_port(default_addr.port());

            to_socket_addr(format!("{}:{}", addr, port).as_str())
        })
    }
    pub fn get_serve_root(&self, default: &str) -> PathBuf {
        PathBuf::from(
            self.serve_root
                .to_owned()
                .unwrap_or_else(|| default.to_string()),
        )
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
    pub fn get_bootstrap(&self) -> &ConfigDefaultsBootstrap {
        match &self.bootstrap {
            Some(x) => &x,
            None => &EMPTY_CONFIG_DEFAULTS_BOOTSTRAP,
        }
    }
    pub fn get_build(&self) -> &ConfigDefaultsBuild {
        match &self.build {
            Some(x) => &x,
            None => &EMPTY_CONFIG_DEFAULTS_BUILD,
        }
    }
    pub fn get_replica(&self) -> &ConfigDefaultsReplica {
        match &self.replica {
            Some(x) => &x,
            None => &EMPTY_CONFIG_DEFAULTS_REPLICA,
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

#[derive(Clone)]
pub struct Config {
    path: PathBuf,
    json: Value,
    // public interface to the config:
    pub config: ConfigInterface,
}

#[allow(dead_code)]
impl Config {
    pub fn resolve_config_path(working_dir: &Path) -> Result<PathBuf, std::io::Error> {
        let mut curr = PathBuf::from(working_dir).canonicalize()?;
        while curr.parent().is_some() {
            if curr.join(CONFIG_FILE_NAME).is_file() {
                return Ok(curr.join(CONFIG_FILE_NAME));
            } else {
                curr.pop();
            }
        }

        // Have to check if the config could be in the root (e.g. on VMs / CI).
        if curr.join(CONFIG_FILE_NAME).is_file() {
            return Ok(curr.join(CONFIG_FILE_NAME));
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Config not found.",
        ))
    }

    pub fn from_file(working_dir: &PathBuf) -> std::io::Result<Config> {
        let path = Config::resolve_config_path(working_dir)?;
        let content = std::fs::read(&path)?;
        Config::from_slice(path, &content)
    }

    pub fn from_current_dir() -> std::io::Result<Config> {
        Config::from_file(&std::env::current_dir()?)
    }

    fn from_slice(path: PathBuf, content: &[u8]) -> std::io::Result<Config> {
        let config = serde_json::from_slice(&content)?;
        let json = serde_json::from_slice(&content)?;
        Ok(Config { path, json, config })
    }

    /// Create a configuration from a string.
    pub fn from_str(content: &str) -> std::io::Result<Config> {
        Config::from_slice(PathBuf::from("-"), content.as_bytes())
    }

    #[cfg(test)]
    pub fn from_str_and_path(path: PathBuf, content: &str) -> std::io::Result<Config> {
        Config::from_slice(path, content.as_bytes())
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

    pub fn get_project_root(&self) -> &Path {
        // a configuration path contains a file name specifically. As
        // such we should be returning at least root as parent. If
        // this is invariance is broken, we must fail.
        self.path.parent().expect(
            "An incorrect configuration path was set with no parent, i.e. did not include root",
        )
    }

    pub fn save(&self) -> DfxResult {
        let json_pretty = serde_json::to_string_pretty(&self.json).or_else(|e| {
            Err(DfxError::InvalidData(format!(
                "Failed to serialize dfx.json: {}",
                e
            )))
        })?;
        std::fs::write(&self.path, json_pretty)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_dfinity_config_current_path() {
        let root_dir = tempfile::tempdir().unwrap();
        let root_path = root_dir.into_path().canonicalize().unwrap();
        let config_path = root_path.join("foo/fah/bar").join(CONFIG_FILE_NAME);

        std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        std::fs::write(&config_path, "{}").unwrap();

        assert_eq!(
            config_path,
            Config::resolve_config_path(config_path.parent().unwrap()).unwrap(),
        );
    }

    #[test]
    fn find_dfinity_config_parent() {
        let root_dir = tempfile::tempdir().unwrap();
        let root_path = root_dir.into_path().canonicalize().unwrap();
        let config_path = root_path.join("foo/fah/bar").join(CONFIG_FILE_NAME);

        std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        std::fs::write(&config_path, "{}").unwrap();

        assert!(
            Config::resolve_config_path(config_path.parent().unwrap().parent().unwrap()).is_err()
        );
    }

    #[test]
    fn find_dfinity_config_subdir() {
        let root_dir = tempfile::tempdir().unwrap();
        let root_path = root_dir.into_path().canonicalize().unwrap();
        let config_path = root_path.join("foo/fah/bar").join(CONFIG_FILE_NAME);
        let subdir_path = config_path.parent().unwrap().join("baz/blue");

        std::fs::create_dir_all(&subdir_path).unwrap();
        std::fs::write(&config_path, "{}").unwrap();

        assert_eq!(
            config_path,
            Config::resolve_config_path(subdir_path.as_path()).unwrap(),
        );
    }

    #[test]
    fn config_defaults_start_addr() {
        let config = Config::from_str(
            r#"{
            "defaults": {
                "start": {
                    "address": "localhost",
                    "port": 8000
                }
            }
        }"#,
        )
        .unwrap();

        assert_eq!(
            config
                .get_config()
                .get_defaults()
                .get_start()
                .get_binding_socket_addr("1.2.3.4:123")
                .ok(),
            to_socket_addr("localhost:8000").ok()
        );
    }

    #[test]
    fn config_defaults_start_addr_no_address() {
        let config = Config::from_str(
            r#"{
            "defaults": {
                "start": {
                    "port": 8000
                }
            }
        }"#,
        )
        .unwrap();

        assert_eq!(
            config
                .get_config()
                .get_defaults()
                .get_start()
                .get_binding_socket_addr("1.2.3.4:123")
                .ok(),
            to_socket_addr("1.2.3.4:8000").ok()
        );
    }

    #[test]
    fn config_defaults_start_addr_no_port() {
        let config = Config::from_str(
            r#"{
            "defaults": {
                "start": {
                    "address": "localhost"
                }
            }
        }"#,
        )
        .unwrap();

        assert_eq!(
            config
                .get_config()
                .get_defaults()
                .get_start()
                .get_binding_socket_addr("1.2.3.4:123")
                .ok(),
            to_socket_addr("localhost:123").ok()
        );
    }
}
