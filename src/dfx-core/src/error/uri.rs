use std::num::ParseIntError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum UriError {
    #[error(transparent)]
    NetworkConfigError(#[from] crate::error::network_config::NetworkConfigError),

    #[error(transparent)]
    FsError(#[from] crate::error::fs::FsError),

    #[error("Failed to read port value from '{0}': {1}")]
    PortReadError(std::path::PathBuf, ParseIntError),

    #[error("Failed to parse url '{0}': {1}")]
    UrlParseError(String, url::ParseError),

    #[error("Failed to determine replica urls: {0}")]
    ReplicaUrlsError(Box<Self>),

    #[error("Failed to get providers for network '{0}': {1}")]
    ProvidersError(String, Box<Self>),
}
