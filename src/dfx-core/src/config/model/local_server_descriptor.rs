use crate::config::model::bitcoin_adapter;
use crate::config::model::canister_http_adapter::HttpAdapterLogLevel;
use crate::config::model::dfinity::{
    to_socket_addr, ConfigDefaultsBitcoin, ConfigDefaultsCanisterHttp, ConfigDefaultsProxy,
    ConfigDefaultsReplica, ReplicaLogLevel, ReplicaSubnetType, DEFAULT_PROJECT_LOCAL_BIND,
    DEFAULT_SHARED_LOCAL_BIND,
};
use crate::config::model::replica_config::CachedConfig;
use crate::error::network_config::{
    NetworkConfigError, NetworkConfigError::ParseBindAddressFailed,
};
use crate::error::structured_file::StructuredFileError;
use crate::json::load_json_file;
use crate::json::structure::SerdeVec;
use serde::{Deserialize, Serialize};
use slog::{debug, info, Logger};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use time::OffsetDateTime;

use super::replica_config::CachedReplicaConfig;

#[derive(Deserialize, Serialize, Debug)]
pub struct NetworkMetadata {
    pub created: OffsetDateTime,
    pub settings_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LocalNetworkScopeDescriptor {
    Project,
    Shared { network_id_path: PathBuf },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalServerDescriptor {
    /// The data directory is one of the following:
    ///     <project directory>/.dfx/network/local
    ///     $HOME/Library/Application Support/org.dfinity.dfx/network/local
    ///     $HOME/.local/share/dfx/network/local
    ///     $APPDATA/dfx/network/local
    pub data_directory: PathBuf,
    pub settings_digest: Option<String>,

    pub bind_address: SocketAddr,

    pub bitcoin: ConfigDefaultsBitcoin,
    pub canister_http: ConfigDefaultsCanisterHttp,
    pub proxy: ConfigDefaultsProxy,
    pub replica: ConfigDefaultsReplica,

    pub scope: LocalNetworkScopeDescriptor,

    legacy_pid_path: Option<PathBuf>,
}

impl LocalNetworkScopeDescriptor {
    pub fn shared(data_directory: &Path) -> Self {
        LocalNetworkScopeDescriptor::Shared {
            network_id_path: data_directory.join("network-id"),
        }
    }
}

impl LocalServerDescriptor {
    pub fn new(
        data_directory: PathBuf,
        bind: String,
        bitcoin: ConfigDefaultsBitcoin,
        canister_http: ConfigDefaultsCanisterHttp,
        proxy: ConfigDefaultsProxy,
        replica: ConfigDefaultsReplica,
        scope: LocalNetworkScopeDescriptor,
        legacy_pid_path: Option<PathBuf>,
    ) -> Result<Self, NetworkConfigError> {
        let settings_digest = None;
        let bind_address = to_socket_addr(&bind).map_err(ParseBindAddressFailed)?;
        Ok(LocalServerDescriptor {
            data_directory,
            settings_digest,
            bind_address,
            bitcoin,
            canister_http,
            proxy,
            replica,
            scope,
            legacy_pid_path,
        })
    }

    /// The contents of this file are different for each `dfx start --clean`
    /// or `dfx start` when the network data directory doesn't already exist
    pub fn network_id_path(&self) -> PathBuf {
        self.data_dir_by_settings_digest().join("network-id")
    }

    /// This file contains the pid of the process started with `dfx start`
    pub fn dfx_pid_path(&self) -> PathBuf {
        self.data_directory.join("pid")
    }

    /// The path of the pid file, as well as one that dfx <= 0.11.x would have created
    pub fn dfx_pid_paths(&self) -> Vec<PathBuf> {
        let mut pid_paths: Vec<PathBuf> = vec![];
        if let Some(legacy_pid_path) = &self.legacy_pid_path {
            pid_paths.push(legacy_pid_path.clone());
        }
        pid_paths.push(self.dfx_pid_path());
        pid_paths
    }

    /// This file contains the configuration/API port of the pocket-ic replica process
    pub fn pocketic_port_path(&self) -> PathBuf {
        self.data_directory.join("pocket-ic-port")
    }

    /// This file contains the pid of the pocket-ic replica process
    pub fn pocketic_pid_path(&self) -> PathBuf {
        self.data_directory.join("pocket-ic-pid")
    }

    /// Returns whether the local server is PocketIC (as opposed to the replica)
    pub fn effective_config(&self) -> Result<Option<CachedConfig<'static>>, StructuredFileError> {
        let path = self.effective_config_path();
        path.exists().then(|| load_json_file(&path)).transpose()
    }

    pub fn load_settings_digest(&mut self) -> Result<(), StructuredFileError> {
        if self.settings_digest.is_none() {
            let network_id_path = self.data_directory.join("network-id");
            let network_metadata: NetworkMetadata = load_json_file(&network_id_path)?;
            self.settings_digest = Some(network_metadata.settings_digest);
        }
        Ok(())
    }

    pub fn settings_digest(&self) -> &str {
        self.settings_digest
            .as_ref()
            .expect("settings_digest must be set")
    }

    pub fn data_dir_by_settings_digest(&self) -> PathBuf {
        if self.scope == LocalNetworkScopeDescriptor::Project {
            self.data_directory.clone()
        } else {
            self.data_directory.join(self.settings_digest())
        }
    }

    /// The top-level directory holding state for the replica.
    pub fn state_dir(&self) -> PathBuf {
        self.data_dir_by_settings_digest().join("state")
    }

    /// The replicated state of the replica.
    pub fn replicated_state_dir(&self) -> PathBuf {
        self.state_dir().join("replicated_state")
    }

    /// This file contains the listening port of the HTTP gateway.
    /// This is the port that the agent connects to.
    pub fn webserver_port_path(&self) -> PathBuf {
        self.data_directory.join("webserver-port")
    }

    /// This file contains the effective config the replica was started with.
    pub fn effective_config_path(&self) -> PathBuf {
        self.data_directory.join("replica-effective-config.json")
    }

    pub fn effective_config_path_by_settings_digest(&self) -> PathBuf {
        self.data_dir_by_settings_digest()
            .join("replica-effective-config.json")
    }
}

impl LocalServerDescriptor {
    pub fn with_bind_address(self, bind_address: SocketAddr) -> Self {
        Self {
            bind_address,
            ..self
        }
    }

    pub fn with_bitcoin_enabled(self) -> LocalServerDescriptor {
        let bitcoin = ConfigDefaultsBitcoin {
            enabled: true,
            ..self.bitcoin
        };
        Self { bitcoin, ..self }
    }

    pub fn with_bitcoin_nodes(self, nodes: Vec<SocketAddr>) -> LocalServerDescriptor {
        let bitcoin = ConfigDefaultsBitcoin {
            nodes: Some(nodes),
            ..self.bitcoin
        };
        Self { bitcoin, ..self }
    }

    pub fn with_proxy_domains(self, domains: Vec<String>) -> LocalServerDescriptor {
        let proxy = ConfigDefaultsProxy {
            domain: Some(SerdeVec::Many(domains)),
        };
        Self { proxy, ..self }
    }

    pub fn with_settings_digest(self, settings_digest: String) -> Self {
        Self {
            settings_digest: Some(settings_digest),
            ..self
        }
    }
}

impl LocalServerDescriptor {
    pub fn describe(&self, log: &Logger) {
        debug!(log, "Local server configuration:");
        let default_bind: SocketAddr = match self.scope {
            LocalNetworkScopeDescriptor::Project => DEFAULT_PROJECT_LOCAL_BIND,
            LocalNetworkScopeDescriptor::Shared { .. } => DEFAULT_SHARED_LOCAL_BIND,
        }
        .parse()
        .unwrap();

        let diffs = if self.bind_address != default_bind {
            format!(" (default: {:?})", default_bind)
        } else {
            "".to_string()
        };
        debug!(log, "  bind address: {:?}{}", self.bind_address, diffs);
        if self.bitcoin.enabled {
            let default_nodes = bitcoin_adapter::default_nodes();
            debug!(log, "  bitcoin: enabled (default: disabled)");
            let nodes: Vec<SocketAddr> = if let Some(ref nodes) = self.bitcoin.nodes {
                nodes.clone()
            } else {
                default_nodes.clone()
            };
            let diffs: String = if nodes != default_nodes {
                format!(" (default: {:?})", default_nodes)
            } else {
                "".to_string()
            };
            debug!(log, "    nodes: {:?}{}", nodes, diffs);
        } else {
            debug!(log, "  bitcoin: disabled");
        }

        if self.canister_http.enabled {
            debug!(log, "  canister http: enabled");
            let diffs: String = if self.canister_http.log_level != HttpAdapterLogLevel::default() {
                format!(" (default: {:?})", HttpAdapterLogLevel::default())
            } else {
                "".to_string()
            };
            debug!(
                log,
                "    log level: {:?}{}", self.canister_http.log_level, diffs
            );
        } else {
            debug!(log, "  canister http: disabled (default: enabled)");
        }

        debug!(log, "  replica:");
        if let Some(port) = self.replica.port {
            debug!(log, "    port: {}", port);
        }
        let subnet_type = self
            .replica
            .subnet_type
            .unwrap_or(ReplicaSubnetType::Application);
        let diffs: String = if subnet_type != ReplicaSubnetType::Application {
            format!(" (default: {:?})", ReplicaSubnetType::Application)
        } else {
            "".to_string()
        };
        debug!(log, "    subnet type: {:?}{}", subnet_type, diffs);

        let log_level = self.replica.log_level.unwrap_or_default();
        let diffs: String = if log_level != ReplicaLogLevel::default() {
            format!(" (default: {:?})", ReplicaLogLevel::default())
        } else {
            "".to_string()
        };
        debug!(log, "    log level: {:?}{}", log_level, diffs);

        debug!(log, "  data directory: {}", self.data_directory.display());
        let scope = match self.scope {
            LocalNetworkScopeDescriptor::Project => "project",
            LocalNetworkScopeDescriptor::Shared { .. } => "shared",
        };
        debug!(log, "  scope: {}", scope);
        debug!(log, "");
    }

    /// Gets the port of a local PocketIC instance.
    ///
    /// # Prerequisites
    /// - A local PocketIC instance needs to be running, e.g. with `dfx start`.
    pub fn get_running_pocketic_port(
        &self,
        logger: Option<&Logger>,
    ) -> Result<Option<u16>, NetworkConfigError> {
        match read_port_from(&self.pocketic_port_path())? {
            Some(port) => {
                if let Some(logger) = logger {
                    info!(logger, "Found local PocketIC running on port {}", port);
                }
                Ok(Some(port))
            }
            None => Ok(None),
        }
    }

    pub fn is_pocketic(&self) -> Result<Option<bool>, StructuredFileError> {
        Ok(self
            .effective_config()?
            .map(|cfg| matches!(cfg.config, CachedReplicaConfig::PocketIc { .. })))
    }
}

/// Reads a port number from a file.
///
/// # Prerequisites
/// The file is expected to contain the port number only, as utf8 text.
fn read_port_from(path: &Path) -> Result<Option<u16>, NetworkConfigError> {
    if path.exists() {
        let s = crate::fs::read_to_string(path)?;
        let s = s.trim();
        if s.is_empty() {
            Ok(None)
        } else {
            let port = s.parse::<u16>().map_err(|e| {
                NetworkConfigError::ParsePortValueFailed(Box::new(path.to_path_buf()), Box::new(e))
            })?;
            Ok(Some(port))
        }
    } else {
        Ok(None)
    }
}
