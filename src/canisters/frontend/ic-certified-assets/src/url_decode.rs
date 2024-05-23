use std::fmt;

use percent_encoding_rfc3986::percent_decode_str;

#[derive(Debug, PartialEq, Eq)]
pub enum UrlDecodeError {
    InvalidPercentEncoding,
}

impl fmt::Display for UrlDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPercentEncoding => write!(f, "invalid percent encoding"),
        }
    }
}

pub fn url_decode(url: &str) -> Result<String, UrlDecodeError> {
    match percent_decode_str(url) {
        Ok(result) => match result.decode_utf8() {
            Ok(result) => Ok(result.to_string()),
            Err(_) => Err(UrlDecodeError::InvalidPercentEncoding),
        },
        Err(_) => Err(UrlDecodeError::InvalidPercentEncoding),
    }
}
