use std::num::ParseIntError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum UriError {
    #[error(transparent)]
    FsError(#[from] crate::error::fs::FsError),

    #[error("Failed to read port value from '{0}': {1}")]
    PortReadError(std::path::PathBuf, ParseIntError),

    #[error("Failed to parse url '{0}': {1}")]
    UrlParseError(String, url::ParseError),
}
