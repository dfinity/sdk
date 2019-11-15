use serde_cbor::error::Error as SerdeError;

#[derive(Debug)]
pub enum AgentError {
    InvalidClientUrl(String),
    IDLSerializationError(),
    IDLDeserializationError(serde_idl::Error),
    InvalidData(serde_cbor::Error),
    EmptyResponse(),
    ClientError(u16, String),

    ReqwestError(reqwest::Error),
    SerdeError(SerdeError),
    UrlParseError(url::ParseError),
}

impl From<SerdeError> for AgentError {
    fn from(err: SerdeError) -> AgentError {
        AgentError::SerdeError(err)
    }
}

impl From<reqwest::Error> for AgentError {
    fn from(err: reqwest::Error) -> AgentError {
        AgentError::ReqwestError(err)
    }
}

impl From<url::ParseError> for AgentError {
    fn from(err: url::ParseError) -> AgentError {
        AgentError::UrlParseError(err)
    }
}
