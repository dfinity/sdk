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
    Wabt(wabt::Error),
    Notify(notify::Error),

    StdIo(std::io::Error),
    StdNumParseIntError(std::num::ParseIntError),

    ClientError(RejectCode, String),
    Unknown(String),
}

/// The result of running a DFX command.
pub type DfxResult<T = ()> = Result<T, DfxError>;

macro_rules! dfx_from_error {
    ($t: ty, $e: ident) => {
        impl From<$t> for DfxError {
            fn from(err: $t) -> DfxError {
                DfxError::$e(err)
            }
        }
    };
}

dfx_from_error!(clap::Error, Clap);
dfx_from_error!(notify::Error, Notify);
dfx_from_error!(reqwest::Error, Reqwest);
dfx_from_error!(reqwest::UrlError, Url);
dfx_from_error!(std::io::Error, StdIo);
dfx_from_error!(std::num::ParseIntError, StdNumParseIntError);
dfx_from_error!(serde_json::error::Error, SerdeJson);
dfx_from_error!(wabt::Error, Wabt);
