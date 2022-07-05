use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;

const BITCOIND_REGTEST_DEFAULT_PORT: u16 = 18444;

/// The btc adapter panics on macos if started within the first (idle seconds+1) seconds
/// after reboot.
/// The current default is 3600 seconds (1 hour).
/// The previous default was 5 seconds, so using that until this issue is fixed in the adapter.
/// See https://dfinity.atlassian.net/browse/SDK-465
const FORCED_IDLE_SECONDS: u64 = 5;

pub fn default_nodes() -> Vec<SocketAddr> {
    vec![SocketAddr::new(
        IpAddr::V4(Ipv4Addr::LOCALHOST),
        BITCOIND_REGTEST_DEFAULT_PORT,
    )]
}

// These definitions come from https://gitlab.com/dfinity-lab/public/ic/-/blob/master/rs/bitcoin/adapter/src/config.rs
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

/// Represents the log level of the bitcoin adapter.
#[derive(Clone, Debug, Serialize, Deserialize, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BitcoinAdapterLogLevel {
    Critical,
    Error,
    Warning,
    Info,
    Debug,
    Trace,
}

impl FromStr for BitcoinAdapterLogLevel {
    type Err = String;

    fn from_str(input: &str) -> Result<BitcoinAdapterLogLevel, Self::Err> {
        match input {
            "critical" => Ok(BitcoinAdapterLogLevel::Critical),
            "error" => Ok(BitcoinAdapterLogLevel::Error),
            "warning" => Ok(BitcoinAdapterLogLevel::Warning),
            "info" => Ok(BitcoinAdapterLogLevel::Info),
            "debug" => Ok(BitcoinAdapterLogLevel::Debug),
            "trace" => Ok(BitcoinAdapterLogLevel::Trace),
            other => Err(format!("Unknown log level: {}", other)),
        }
    }
}

impl Default for BitcoinAdapterLogLevel {
    fn default() -> Self {
        BitcoinAdapterLogLevel::Info
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LoggerConfig {
    level: BitcoinAdapterLogLevel,
}

/// This struct contains configuration options for the BTC Adapter.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    /// The type of Bitcoin network we plan to communicate to (e.g. "mainnet", "testnet", "regtest", etc.).
    pub network: String,
    /// Addresses of nodes to connect to (in case discovery from seeds is not possible/sufficient)
    #[serde(default)]
    pub nodes: Vec<SocketAddr>,
    /// The number of seconds that need to pass for the adapter to enter the
    /// `Idle` state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idle_seconds: Option<u64>,
    /// Specifies which unix domain socket should be used for serving incoming requests.
    #[serde(default)]
    pub incoming_source: IncomingSource,

    pub logger: LoggerConfig,
}

impl Config {
    pub fn new(
        nodes: Vec<SocketAddr>,
        uds_path: PathBuf,
        log_level: BitcoinAdapterLogLevel,
    ) -> Config {
        Config {
            network: String::from("regtest"),
            nodes,
            idle_seconds: Some(FORCED_IDLE_SECONDS),
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
