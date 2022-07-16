use crate::config::dfinity::to_socket_addr;
use crate::config::dfinity::{
    ConfigDefaultsBitcoin, ConfigDefaultsBootstrap, ConfigDefaultsCanisterHttp,
    ConfigDefaultsReplica,
};
use crate::lib::error::DfxResult;

use anyhow::Context;
use fn_error_context::context;
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub struct LocalServerDescriptor {
    /// The data directory is <project directory>/.dfx
    pub data_directory: PathBuf,

    pub bind_address: SocketAddr,

    pub bitcoin: ConfigDefaultsBitcoin,
    pub bootstrap: ConfigDefaultsBootstrap,
    pub canister_http: ConfigDefaultsCanisterHttp,
    pub replica: ConfigDefaultsReplica,
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
        })
    }

    /// This file contains the pid of the process started with `dfx start`
    pub fn dfx_pid_path(&self) -> PathBuf {
        self.data_directory.join("pid")
    }

    /// This file contains the pid of the icx-proxy process
    pub fn icx_proxy_pid_path(&self) -> PathBuf {
        self.data_directory.join("icx-proxy-pid")
    }

    /// This file contains the port of the internal candid webserver, but only for `dfx bootstrap`
    pub fn proxy_port_path(&self) -> PathBuf {
        self.data_directory.join("proxy-port")
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
