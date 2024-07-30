use crate::error::config::{ConfigError, GetTempPathError};
use crate::error::fs::ReadToStringError;
use crate::error::socket_addr_conversion::SocketAddrConversionError;
use crate::error::uri::UriError;

use candid::types::principal::PrincipalError;
use std::num::ParseIntError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NetworkConfigError {
    #[error(transparent)]
    FsError(#[from] crate::error::fs::FsError),

    #[error(transparent)]
    ReadToString(#[from] ReadToStringError),

    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error(transparent)]
    UriError(#[from] UriError),

    #[error("Failed to get replica endpoint for network '{network_name}'")]
    GettingReplicaUrlsFailed {
        network_name: String,
        source: UriError,
    },

    #[error(transparent)]
    GetTempPath(#[from] GetTempPathError),

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

    #[error("Failed to parse bind address")]
    ParseBindAddressFailed(#[source] SocketAddrConversionError),

    #[error("Failed to parse contents of {0} as a port value")]
    ParsePortValueFailed(Box<PathBuf>, #[source] Box<ParseIntError>),

    #[error("Failed to parse URL '{0}'")]
    ParseProviderUrlFailed(Box<String>, #[source] url::ParseError),

    #[error("Failed to read webserver port")]
    ReadWebserverPortFailed(#[source] ReadToStringError),

    #[error("Failed to parse principal '{0}'")]
    ParsePrincipalFailed(String, #[source] PrincipalError),
}
