use crate::RequestIdError;
use serde_cbor::error::Error as SerdeError;

#[derive(Debug)]
pub enum AgentError {
    InvalidClientUrl(String),
    InvalidClientResponse,
    CannotCalculateRequestId(RequestIdError),
    EmptyResponse(),
    ClientError(u16, String),
    TimeoutWaitingForResponse,

    SigningError(String),

    InvalidCborData(serde_cbor::Error),
    ReqwestError(reqwest::Error),
    SerdeError(SerdeError),
    UrlParseError(url::ParseError),
}

impl From<SerdeError> for AgentError {
    fn from(err: SerdeError) -> Self {
        Self::SerdeError(err)
    }
}

impl From<reqwest::Error> for AgentError {
    fn from(err: reqwest::Error) -> Self {
        Self::ReqwestError(err)
    }
}

impl From<url::ParseError> for AgentError {
    fn from(err: url::ParseError) -> Self {
        Self::UrlParseError(err)
    }
}

impl From<RequestIdError> for AgentError {
    fn from(err: RequestIdError) -> Self {
        Self::CannotCalculateRequestId(err)
    }
}
