use std::num::ParseIntError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum UriError {
    #[error(transparent)]
    NetworkConfigError(#[from] crate::error::network_config::NetworkConfigError),
    #[error(transparent)]
    FsError(#[from] crate::error::fs::FsError),
    #[error("Failed to read port value from '{0}': {1}")]
    PortReadError(String, ParseIntError),
    // next error should accept a url::ParseError
    #[error("Failed to parse url '{0}': {1}")]
    UrlParseError(String, url::ParseError),
    #[error("Failed to determine replica urls: {0}")]
    ReplicaUrlsError(String),
    #[error("Failed to get providers for network '{0}': {1}")]
    ProvidersError(String, Box<Self>),
}
