use crate::error::config::ConfigError;
use crate::error::fs::FsError;
use crate::error::socket_addr_conversion::SocketAddrConversionError;
use crate::error::uri::UriError;
use std::num::ParseIntError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NetworkConfigError {
    #[error(transparent)]
    FsError(#[from] crate::error::fs::FsError),

    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error(transparent)]
    UriError(#[from] UriError),

    #[error("Failed to get replica endpoint for network '{network_name}': {cause}")]
    GettingReplicaUrlsFailed {
        network_name: String,
        cause: UriError,
    },

    #[error("Network '{0}' does not specify any network providers.")]
    NetworkHasNoProviders(String),

    #[error("The '{0}' network must be a local network.")]
    NetworkMustBeLocal(String),

    #[error("Network not found: {0}")]
    NetworkNotFound(String),

    #[error("Cannot find network context.")]
    NoNetworkContext(),

    #[error("Did not find any providers for network '{0}'")]
    NoProvidersForNetwork(String),

    #[error("Failed to parse bind address: {0}")]
    ParseBindAddressFailed(SocketAddrConversionError),

    #[error("Failed to parse contents of {0} as a port value: {1}")]
    ParsePortValueFailed(Box<PathBuf>, Box<ParseIntError>),

    #[error("Failed to parse URL '{0}': {1}")]
    ParseProviderUrlFailed(Box<String>, url::ParseError),

    #[error("Failed to read webserver port: {0}")]
    ReadWebserverPortFailed(FsError),
}
