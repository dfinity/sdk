use thiserror::Error;

#[derive(Error, Debug)]
pub enum UriError {
    #[error("Failed to parse url '{0}'")]
    UrlParseError(String, #[source] url::ParseError),
}
