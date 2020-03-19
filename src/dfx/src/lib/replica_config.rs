use crate::lib::error::{DfxError, DfxResult};

use serde::{Deserialize, Serialize};
use std::default::Default;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HttpHandlerConfig {
    /// Instructs the HTTP handler to use the specified port
    pub use_port: Option<u16>,

    /// Instructs the HTTP handler to bind to any open port and report the port
    /// to the specified file.
    /// The port is written in its textual representation, no newline at the
    /// end.
    pub write_port_to: Option<PathBuf>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SchedulerConfig {
    pub exec_gas: Option<u64>,
    pub round_gas_max: Option<u64>,
}

impl SchedulerConfig {
    pub fn validate(self) -> DfxResult<Self> {
        if self.exec_gas >= self.round_gas_max {
            let message = "Round gas limit must exceed message gas limit.";
            Err(DfxError::InvalidData(message.to_string()))
        } else {
            Ok(self)
        }
    }
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
    pub scheduler: SchedulerConfig,
    pub state_manager: StateManagerConfig,
    pub crypto: CryptoConfig,
    pub artifact_pool: ArtifactPoolConfig,
}

impl ReplicaConfig {
    pub fn new(state_root: PathBuf) -> Self {
        ReplicaConfig {
            http_handler: HttpHandlerConfig {
                write_port_to: None,
                use_port: None,
            },
            scheduler: SchedulerConfig {
                exec_gas: None,
                round_gas_max: None,
            },
            state_manager: StateManagerConfig {
                state_root: state_root.join("ic_state"),
            },
            crypto: CryptoConfig {
                crypto_root: state_root.join("ic_crypto"),
            },
            artifact_pool: ArtifactPoolConfig {
                consensus_pool_path: state_root.join("ic_consensus"),
            },
        }
    }

    #[allow(dead_code)]
    pub fn with_port(&mut self, port: u16) -> &mut Self {
        self.http_handler.use_port = Some(port);
        self.http_handler.write_port_to = None;
        self
    }

    pub fn with_random_port(&mut self, write_port_to: &Path) -> &mut Self {
        self.http_handler.use_port = None;
        self.http_handler.write_port_to = Some(write_port_to.to_path_buf());
        self
    }

    pub fn to_toml(&self) -> DfxResult<String> {
        toml::to_string(&self).map_err(DfxError::CouldNotSerializeClientConfiguration)
    }
}
