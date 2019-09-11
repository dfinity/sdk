use crate::lib::api_client::RejectCode;

// TODO: refactor this enum into a *Kind enum and a struct DfxError.
#[derive(Debug)]
pub enum DfxError {
    Clap(clap::Error),
    IO(std::io::Error),
    ParseInt(std::num::ParseIntError),
    Reqwest(reqwest::Error),
    SerdeCbor(serde_cbor::error::Error),
    SerdeJson(serde_json::error::Error),
    Url(reqwest::UrlError),

    /// An unknown command was used. The argument is the command itself.
    UnknownCommand(String),

    ClientError(RejectCode, String),
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
