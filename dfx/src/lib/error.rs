// use crate::lib::api_client::ReadRejectCode;
use ic_http_agent::{RequestIdError, RequestIdFromStringError};

#[derive(Debug)]
pub enum BuildErrorKind {
    InvalidExtension(String),
    CompilerError,
}

// TODO: refactor this enum into a *Kind enum and a struct DfxError.
#[derive(Debug)]
pub enum DfxError {
    BuildError(BuildErrorKind),
    Clap(clap::Error),
    IO(std::io::Error),
    ParseInt(std::num::ParseIntError),
    Reqwest(reqwest::Error),
    SerdeCborFromServer(serde_cbor::error::Error, String),
    SerdeCbor(serde_cbor::error::Error),
    SerdeJson(serde_json::error::Error),
    Url(reqwest::UrlError),
    WabtError(wabt::Error),
    AddrParseError(std::net::AddrParseError),
    HttpAgentError(RequestIdError),
    RequestIdFromStringError(RequestIdFromStringError),

    /// An unknown command was used. The argument is the command itself.
    UnknownCommand(String),

    // Cannot create a new project because the directory already exists.
    ProjectExists(),

    ClientError(u16, String),
    Unknown(String),
}

/// The result of running a DFX command.
pub type DfxResult<T = ()> = Result<T, DfxError>;

impl From<clap::Error> for DfxError {
    fn from(err: clap::Error) -> DfxError {
        DfxError::Clap(err)
    }
}

impl From<reqwest::Error> for DfxError {
    fn from(err: reqwest::Error) -> DfxError {
        DfxError::Reqwest(err)
    }
}

impl From<reqwest::UrlError> for DfxError {
    fn from(err: reqwest::UrlError) -> DfxError {
        DfxError::Url(err)
    }
}

impl From<serde_cbor::Error> for DfxError {
    fn from(err: serde_cbor::Error) -> DfxError {
        DfxError::SerdeCbor(err)
    }
}

impl From<serde_json::Error> for DfxError {
    fn from(err: serde_json::Error) -> DfxError {
        DfxError::SerdeJson(err)
    }
}

impl From<std::io::Error> for DfxError {
    fn from(err: std::io::Error) -> DfxError {
        DfxError::IO(err)
    }
}

impl From<std::num::ParseIntError> for DfxError {
    fn from(err: std::num::ParseIntError) -> DfxError {
        DfxError::ParseInt(err)
    }
}

impl From<wabt::Error> for DfxError {
    fn from(err: wabt::Error) -> DfxError {
        DfxError::WabtError(err)
    }
}

impl From<std::net::AddrParseError> for DfxError {
    fn from(err: std::net::AddrParseError) -> DfxError {
        DfxError::AddrParseError(err)
    }
}

impl From<RequestIdError> for DfxError {
    fn from(err: RequestIdError) -> DfxError {
        DfxError::HttpAgentError(err)
    }
}

impl From<RequestIdFromStringError> for DfxError {
    fn from(err: RequestIdFromStringError) -> DfxError {
        DfxError::RequestIdFromStringError(err)
    }
}
