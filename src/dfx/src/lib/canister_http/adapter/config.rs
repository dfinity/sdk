use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, str::FromStr};

// These definitions come from https://gitlab.com/dfinity-lab/public/ic/-/blob/master/rs/canister_http/adapter/src/config.rs
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
/// The source of the unix domain socket to be used for inter-process
/// communication.
pub enum IncomingSource {
    /// We use systemd's created socket.
    Systemd,
    /// We use the corresponing path as socket.
    Path(PathBuf),
}

impl Default for IncomingSource {
    fn default() -> Self {
        IncomingSource::Systemd
    }
}

/// Represents the log level of the HTTP adapter.
#[derive(Clone, Debug, Serialize, Deserialize, Copy, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HttpAdapterLogLevel {
    Critical,
    Error,
    Warning,
    Info,
    Debug,
    Trace,
}

impl FromStr for HttpAdapterLogLevel {
    type Err = String;

    fn from_str(input: &str) -> Result<HttpAdapterLogLevel, Self::Err> {
        match input {
            "critical" => Ok(HttpAdapterLogLevel::Critical),
            "error" => Ok(HttpAdapterLogLevel::Error),
            "warning" => Ok(HttpAdapterLogLevel::Warning),
            "info" => Ok(HttpAdapterLogLevel::Info),
            "debug" => Ok(HttpAdapterLogLevel::Debug),
            "trace" => Ok(HttpAdapterLogLevel::Trace),
            other => Err(format!("Unknown log level: {}", other)),
        }
    }
}

impl Default for HttpAdapterLogLevel {
    fn default() -> Self {
        HttpAdapterLogLevel::Error
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LoggerConfig {
    pub level: HttpAdapterLogLevel,
}

/// This struct contains configuration options for the Canister HTTP Adapter.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    /// Specifies which unix domain socket should be used for serving incoming requests.
    #[serde(default)]
    pub incoming_source: IncomingSource,

    pub logger: LoggerConfig,
}

impl Config {
    pub fn new(uds_path: PathBuf, log_level: HttpAdapterLogLevel) -> Config {
        Config {
            incoming_source: IncomingSource::Path(uds_path),
            logger: LoggerConfig { level: log_level },
        }
    }

    pub fn get_socket_path(&self) -> Option<PathBuf> {
        match &self.incoming_source {
            IncomingSource::Systemd => None,
            IncomingSource::Path(path) => Some(path.clone()),
        }
    }
}
