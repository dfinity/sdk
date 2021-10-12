use serde::{Deserialize, Serialize};
use std::default::Default;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HttpHandlerConfig {
    /// Instructs the HTTP handler to use the specified port
    pub port: Option<u16>,

    /// Instructs the HTTP handler to bind to any open port and report the port
    /// to the specified file.
    /// The port is written in its textual representation, no newline at the
    /// end.
    pub write_port_to: Option<PathBuf>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ArtifactPoolConfig {
    pub consensus_pool_path: PathBuf,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CryptoConfig {
    pub crypto_root: PathBuf,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StateManagerConfig {
    pub state_root: PathBuf,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ReplicaConfig {
    pub http_handler: HttpHandlerConfig,
    pub state_manager: StateManagerConfig,
    pub crypto: CryptoConfig,
    pub artifact_pool: ArtifactPoolConfig,
}

impl ReplicaConfig {
    pub fn new(state_root: &Path) -> Self {
        ReplicaConfig {
            http_handler: HttpHandlerConfig {
                write_port_to: None,
                port: None,
            },
            state_manager: StateManagerConfig {
                state_root: state_root.join("replicated_state"),
            },
            crypto: CryptoConfig {
                crypto_root: state_root.join("crypto_store"),
            },
            artifact_pool: ArtifactPoolConfig {
                consensus_pool_path: state_root.join("consensus_pool"),
            },
        }
    }

    #[allow(dead_code)]
    pub fn with_port(&mut self, port: u16) -> &mut Self {
        self.http_handler.port = Some(port);
        self.http_handler.write_port_to = None;
        self
    }

    pub fn with_random_port(&mut self, write_port_to: &Path) -> Self {
        self.http_handler.port = None;
        self.http_handler.write_port_to = Some(write_port_to.to_path_buf());
        let config = &*self;
        config.clone()
    }
}
