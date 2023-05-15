use thiserror::Error;

#[derive(Error, Debug)]
pub enum UriError {
    #[error("Failed to parse url '{0}': {1}")]
    UrlParseError(String, url::ParseError),
}
