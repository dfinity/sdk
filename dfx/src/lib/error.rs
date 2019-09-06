use crate::lib::api_client::RejectCode;

// TODO: refactor this enum into a *Kind enum and a struct DfxError.
#[derive(Debug)]
pub enum DfxError {
    Clap(clap::Error),
    Reqwest(reqwest::Error),
    SerdeCbor(serde_cbor::error::Error),
    SerdeJson(serde_json::error::Error),
    Url(reqwest::UrlError),

    UnknownCommand(String),

    StdIo(std::io::Error),
    StdNumParseIntError(std::num::ParseIntError),

    ClientError(RejectCode, String),
    Unknown(String),
}

/// The result of running a DFX command.
pub type DfxResult = Result<(), DfxError>;

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

impl From<clap::Error> for DfxError {
    fn from(err: clap::Error) -> DfxError {
        DfxError::Clap(err)
    }
}

impl From<std::io::Error> for DfxError {
    fn from(err: std::io::Error) -> DfxError {
        DfxError::StdIo(err)
    }
}

impl From<serde_json::error::Error> for DfxError {
    fn from(err: serde_json::error::Error) -> DfxError {
        DfxError::SerdeJson(err)
    }
}

impl From<std::num::ParseIntError> for DfxError {
    fn from(err: std::num::ParseIntError) -> DfxError {
        DfxError::StdNumParseIntError(err)
    }
}
