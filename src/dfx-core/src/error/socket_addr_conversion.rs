use std::net::AddrParseError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SocketAddrConversionError {
    #[error("Failed to convert {0} to a socket address: {1}")]
    ParseSocketAddrFailed(String, AddrParseError),
}
