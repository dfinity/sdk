use std::fmt;

use percent_encoding::percent_decode_str;

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

/// Decodes a percent encoded string according to https://url.spec.whatwg.org/#percent-decode
///
/// This is a wrapper around the percent-encoding crate.
///
/// The rules that it follow by are:
/// - Start with an empty sequence of bytes of the output
/// - Convert the input to a sequence of bytes
/// - if the byte is `%` and the next two bytes are hex, convet the hex value to a byte
///   and add it to the output, otherwise add the byte to the output
/// - convert the output byte sequence to a UTF-8 string and return it. If the conversion
///   fails return an error.
pub fn url_decode(url: &str) -> Result<String, UrlDecodeError> {
    match percent_decode_str(url).decode_utf8() {
        Ok(result) => Ok(result.to_string()),
        Err(_) => Err(UrlDecodeError::InvalidPercentEncoding),
    }
}
