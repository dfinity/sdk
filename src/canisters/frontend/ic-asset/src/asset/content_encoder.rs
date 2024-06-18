use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Copy, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ContentEncoder {
    Gzip,
    #[serde(alias = "br")]
    Brotli,
    #[serde(alias = "id")]
    Identity,
}

impl std::fmt::Display for ContentEncoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            ContentEncoder::Gzip => f.write_str("gzip"),
            ContentEncoder::Brotli => f.write_str("br"),
            ContentEncoder::Identity => f.write_str("identity"),
        }
    }
}
