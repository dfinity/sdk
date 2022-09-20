use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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

/// This struct contains configuration options for the Canister HTTP Adapter.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    /// Specifies which unix domain socket should be used for serving incoming requests.
    #[serde(default)]
    pub incoming_source: IncomingSource,
}

impl Config {
    pub fn new(uds_path: PathBuf) -> Config {
        Config {
            incoming_source: IncomingSource::Path(uds_path),
        }
    }

    pub fn get_socket_path(&self) -> Option<PathBuf> {
        match &self.incoming_source {
            IncomingSource::Systemd => None,
            IncomingSource::Path(path) => Some(path.clone()),
        }
    }
}
