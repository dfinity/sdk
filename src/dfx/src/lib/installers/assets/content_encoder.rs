use crate::lib::error::DfxResult;

use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Write;

pub enum ContentEncoder {
    Gzip,
    Identity,
}

impl ContentEncoder {
    pub fn encode(&self, content: &[u8]) -> DfxResult<Vec<u8>> {
        match &self {
            ContentEncoder::Gzip => encode_gzip(content),
            ContentEncoder::Identity => {
                unreachable!("Do not encode for identity because it would copy")
            }
        }
    }
}

impl std::fmt::Display for ContentEncoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            ContentEncoder::Gzip => f.write_str("gzip"),
            ContentEncoder::Identity => f.write_str("identity"),
        }
    }
}

fn encode_gzip(content: &[u8]) -> DfxResult<Vec<u8>> {
    let mut e = GzEncoder::new(Vec::new(), Compression::default());
    e.write_all(content)?;
    let encoded = e.finish()?;
    Ok(encoded)
}
