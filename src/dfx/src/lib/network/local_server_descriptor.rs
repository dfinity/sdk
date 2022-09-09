use crate::config::dfinity::{
    to_socket_addr, ReplicaLogLevel, ReplicaSubnetType, DEFAULT_PROJECT_LOCAL_BIND,
    DEFAULT_SHARED_LOCAL_BIND,
};
use crate::config::dfinity::{
    ConfigDefaultsBitcoin, ConfigDefaultsBootstrap, ConfigDefaultsCanisterHttp,
    ConfigDefaultsReplica,
};
use crate::lib::error::DfxResult;

use anyhow::Context;
use fn_error_context::context;
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq)]
pub enum LocalNetworkScopeDescriptor {
    Project,
    Shared { network_id_path: PathBuf },
}

#[derive(Clone, Debug, PartialEq)]
pub struct LocalServerDescriptor {
    /// The data directory is one of the following:
    ///     <project directory>/.dfx/network/local
    ///     $HOME/Library/Application Support/org.dfinity.dfx/network/local
    ///     $HOME/.local/share/dfx/network/local
    pub data_directory: PathBuf,

    pub bind_address: SocketAddr,

    pub bitcoin: ConfigDefaultsBitcoin,
    pub bootstrap: ConfigDefaultsBootstrap,
    pub canister_http: ConfigDefaultsCanisterHttp,
    pub replica: ConfigDefaultsReplica,

    pub scope: LocalNetworkScopeDescriptor,
}

impl LocalNetworkScopeDescriptor {
    pub fn shared(data_directory: &Path) -> Self {
        LocalNetworkScopeDescriptor::Shared {
            network_id_path: data_directory.join("network-id"),
        }
    }
}

impl LocalServerDescriptor {
    #[context("Failed to construct local server descriptor.")]
    pub(crate) fn new(
        data_directory: PathBuf,
        bind: String,
        bitcoin: ConfigDefaultsBitcoin,
        bootstrap: ConfigDefaultsBootstrap,
        canister_http: ConfigDefaultsCanisterHttp,
        replica: ConfigDefaultsReplica,
        scope: LocalNetworkScopeDescriptor,
    ) -> DfxResult<Self> {
        let bind_address =
            to_socket_addr(&bind).context("Failed to convert 'bind' field to a SocketAddress")?;
        Ok(LocalServerDescriptor {
            data_directory,
            bind_address,
            bitcoin,
            bootstrap,
            canister_http,
            replica,
            scope,
        })
    }

    /// The contents of this file are different for each `dfx start --clean`
    /// or `dfx start` when the network data directory doesn't already exist
    pub fn network_id_path(&self) -> PathBuf {
        self.data_directory.join("network-id")
    }

    /// This file contains the pid of the process started with `dfx start`
    pub fn dfx_pid_path(&self) -> PathBuf {
        self.data_directory.join("pid")
    }

    /// This file contains the pid of the icx-proxy process
    pub fn icx_proxy_pid_path(&self) -> PathBuf {
        self.data_directory.join("icx-proxy-pid")
    }

    /// This file contains the listening port of the ic-ref process
    pub fn ic_ref_port_path(&self) -> PathBuf {
        self.data_directory.join("ic-ref.port")
    }

    /// This file contains the pid of the ic-btc-adapter process
    pub fn btc_adapter_pid_path(&self) -> PathBuf {
        self.data_directory.join("ic-btc-adapter-pid")
    }

    /// This file contains the configuration for the ic-btc-adapter
    pub fn btc_adapter_config_path(&self) -> PathBuf {
        self.data_directory.join("ic-btc-adapter-config.json")
    }

    /// This file contains the PATH of the unix domain socket for the ic-btc-adapter
    pub fn btc_adapter_socket_holder_path(&self) -> PathBuf {
        self.data_directory.join("ic-btc-adapter-socket-path")
    }

    /// This file contains the configuration for the ic-canister-http-adapter
    pub fn canister_http_adapter_config_path(&self) -> PathBuf {
        self.data_directory.join("ic-canister-http-config.json")
    }

    /// This file contains the pid of the ic-canister-http-adapter process
    pub fn canister_http_adapter_pid_path(&self) -> PathBuf {
        self.data_directory.join("ic-canister-http-adapter-pid")
    }

    /// This file contains the PATH of the unix domain socket for the ic-canister-http-adapter
    pub fn canister_http_adapter_socket_holder_path(&self) -> PathBuf {
        self.data_directory.join("ic-canister-http-socket-path")
    }

    /// The replica configuration directory doesn't actually contain replica configuration.
    /// It contains two files:
    ///   - replica-1.port  contains the listening port of the running replica process
    ///   - replica.pid     contains the pid of the running replica process
    pub fn replica_configuration_dir(&self) -> PathBuf {
        self.data_directory.join("replica-configuration")
    }

    /// This file contains the listening port of the replica
    pub fn replica_port_path(&self) -> PathBuf {
        self.replica_configuration_dir().join("replica-1.port")
    }

    /// This file contains the pid of the replica process
    pub fn replica_pid_path(&self) -> PathBuf {
        self.replica_configuration_dir().join("replica-pid")
    }

    /// The top-level directory holding state for the replica.
    pub fn state_dir(&self) -> PathBuf {
        self.data_directory.join("state")
    }

    /// The replicated state of the replica.
    pub fn replicated_state_dir(&self) -> PathBuf {
        self.state_dir().join("replicated_state")
    }

    /// This file contains the listening port of the icx-proxy.
    /// This is the port that the agent connects to.
    pub fn webserver_port_path(&self) -> PathBuf {
        self.data_directory.join("webserver-port")
    }
}

impl LocalServerDescriptor {
    pub(crate) fn with_bind_address(self, bind_address: SocketAddr) -> Self {
        Self {
            bind_address,
            ..self
        }
    }

    pub(crate) fn with_replica_port(self, port: u16) -> Self {
        let replica = ConfigDefaultsReplica {
            port: Some(port),
            ..self.replica
        };
        Self { replica, ..self }
    }

    pub(crate) fn with_bitcoin_enabled(self) -> LocalServerDescriptor {
        let bitcoin = ConfigDefaultsBitcoin {
            enabled: true,
            ..self.bitcoin
        };
        Self { bitcoin, ..self }
    }

    pub(crate) fn with_bitcoin_nodes(self, nodes: Vec<SocketAddr>) -> LocalServerDescriptor {
        let bitcoin = ConfigDefaultsBitcoin {
            nodes: Some(nodes),
            ..self.bitcoin
        };
        Self { bitcoin, ..self }
    }

    pub(crate) fn with_bootstrap_ip(self, ip: IpAddr) -> LocalServerDescriptor {
        let bootstrap = ConfigDefaultsBootstrap {
            ip,
            ..self.bootstrap
        };
        Self { bootstrap, ..self }
    }

    pub(crate) fn with_bootstrap_port(self, port: u16) -> LocalServerDescriptor {
        let bootstrap = ConfigDefaultsBootstrap {
            port,
            ..self.bootstrap
        };
        Self { bootstrap, ..self }
    }

    pub(crate) fn with_bootstrap_timeout(self, timeout: u64) -> LocalServerDescriptor {
        let bootstrap = ConfigDefaultsBootstrap {
            timeout,
            ..self.bootstrap
        };
        Self { bootstrap, ..self }
    }
}

impl LocalServerDescriptor {
    pub fn describe(&self, include_replica: bool, include_replica_port: bool) {
        println!("Local server configuration:");
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
        println!("  bind address: {:?}{}", self.bind_address, diffs);
        if self.bitcoin.enabled {
            let default_nodes = crate::lib::bitcoin::adapter::config::default_nodes();
            println!("  bitcoin: enabled (default: disabled)");
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
            println!("    nodes: {:?}{}", nodes, diffs);
        } else {
            println!("  bitcoin: disabled");
        }

        if self.canister_http.enabled {
            println!("  canister http: enabled");
        } else {
            println!("  canister http: disabled (default: enabled)");
        }

        if include_replica {
            println!("  replica:");
            if include_replica_port {
                println!("    port: ");
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
            println!("    subnet type: {:?}{}", subnet_type, diffs);

            let log_level = self.replica.log_level.unwrap_or_default();
            let diffs: String = if log_level != ReplicaLogLevel::default() {
                format!(" (default: {:?})", ReplicaLogLevel::default())
            } else {
                "".to_string()
            };
            println!("    log level: {:?}{}", log_level, diffs);
        }
        println!("  data directory: {}", self.data_directory.display());
        let scope = match self.scope {
            LocalNetworkScopeDescriptor::Project => "project",
            LocalNetworkScopeDescriptor::Shared { .. } => "shared",
        };
        println!("  scope: {}", scope);
        println!();
    }

    pub fn describe_bootstrap(&self) {
        println!("Bootstrap configuration:");
        let default: ConfigDefaultsBootstrap = Default::default();
        let diffs = if self.bootstrap.ip != default.ip {
            format!("  (default: {:?})", default.ip)
        } else {
            "".to_string()
        };
        println!("  ip: {:?}{}", self.bootstrap.ip, diffs);

        let diffs = if self.bootstrap.port != default.port {
            format!("  (default: {})", default.port)
        } else {
            "".to_string()
        };
        println!("  port: {}{}", self.bootstrap.port, diffs);

        let diffs = if self.bootstrap.timeout != default.timeout {
            format!("  (default: {})", default.timeout)
        } else {
            "".to_string()
        };
        println!("  timeout: {}{}", self.bootstrap.timeout, diffs);
        println!();
    }
}
