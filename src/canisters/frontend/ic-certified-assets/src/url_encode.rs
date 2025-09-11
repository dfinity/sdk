use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};

/// Encodes a percent encoded string according to https://url.spec.whatwg.org/#percent-encode
///
/// This is a wrapper around the percent-encoding crate.
pub fn url_encode(url: &str) -> String {
    utf8_percent_encode(url, NON_ALPHANUMERIC).to_string()
}
