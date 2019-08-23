#[derive(Debug)]
pub enum DfxError {
    Reqwest(reqwest::Error),
    SerdeCbor(serde_cbor::error::Error),
    Url(reqwest::UrlError),
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
