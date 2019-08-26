use crate::lib::api_client::RejectCode;

// TODO: refactor this enum into a *Kind enum and a struct DfxError.
#[derive(Debug)]
pub enum DfxError {
    Clap(clap::Error),
    Reqwest(reqwest::Error),
    SerdeCbor(serde_cbor::error::Error),
    Url(reqwest::UrlError),

    UnknownCommand(String),

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
