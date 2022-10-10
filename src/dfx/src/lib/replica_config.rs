use serde::{Deserialize, Serialize};
use std::default::Default;
use std::path::{Path, PathBuf};

use crate::config::dfinity::{ReplicaLogLevel, ReplicaSubnetType};

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
pub struct BtcAdapterConfig {
    pub enabled: bool,
    pub socket_path: Option<PathBuf>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CanisterHttpAdapterConfig {
    pub enabled: bool,
    pub socket_path: Option<PathBuf>,
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
    pub subnet_type: ReplicaSubnetType,
    pub btc_adapter: BtcAdapterConfig,
    pub canister_http_adapter: CanisterHttpAdapterConfig,
    pub log_level: ReplicaLogLevel,
}

impl ReplicaConfig {
    pub fn new(
        state_root: &Path,
        subnet_type: ReplicaSubnetType,
        log_level: ReplicaLogLevel,
    ) -> Self {
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
            subnet_type,
            btc_adapter: BtcAdapterConfig {
                enabled: false,
                socket_path: None,
            },
            canister_http_adapter: CanisterHttpAdapterConfig {
                enabled: false,
                socket_path: None,
            },
            log_level,
        }
    }

    #[allow(dead_code)]
    pub fn with_port(self, port: u16) -> Self {
        ReplicaConfig {
            http_handler: self.http_handler.with_port(port),
            ..self
        }
    }

    pub fn with_random_port(self, write_port_to: &Path) -> Self {
        ReplicaConfig {
            http_handler: self.http_handler.with_random_port(write_port_to),
            ..self
        }
    }

    pub fn with_btc_adapter_enabled(self) -> Self {
        ReplicaConfig {
            btc_adapter: self.btc_adapter.with_enabled(),
            ..self
        }
    }

    pub fn with_btc_adapter_socket(self, socket_path: PathBuf) -> Self {
        ReplicaConfig {
            btc_adapter: self.btc_adapter.with_socket_path(socket_path),
            ..self
        }
    }

    pub fn with_canister_http_adapter_enabled(self) -> Self {
        ReplicaConfig {
            canister_http_adapter: self.canister_http_adapter.with_enabled(),
            ..self
        }
    }
    pub fn with_canister_http_adapter_socket(self, socket_path: PathBuf) -> Self {
        ReplicaConfig {
            canister_http_adapter: self.canister_http_adapter.with_socket_path(socket_path),
            ..self
        }
    }
}

impl BtcAdapterConfig {
    pub fn with_enabled(self) -> Self {
        BtcAdapterConfig {
            enabled: true,
            ..self
        }
    }

    pub fn with_socket_path(self, socket_path: PathBuf) -> Self {
        BtcAdapterConfig {
            socket_path: Some(socket_path),
            ..self
        }
    }
}

impl CanisterHttpAdapterConfig {
    pub fn with_enabled(self) -> Self {
        CanisterHttpAdapterConfig {
            enabled: true,
            ..self
        }
    }

    pub fn with_socket_path(self, socket_path: PathBuf) -> Self {
        CanisterHttpAdapterConfig {
            socket_path: Some(socket_path),
            ..self
        }
    }
}

impl HttpHandlerConfig {
    pub fn with_port(self, port: u16) -> Self {
        HttpHandlerConfig {
            port: Some(port),
            write_port_to: None,
        }
    }

    pub fn with_random_port(self, write_port_to: &Path) -> Self {
        HttpHandlerConfig {
            port: None,
            write_port_to: Some(write_port_to.to_path_buf()),
        }
    }
}
