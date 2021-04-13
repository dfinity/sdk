use crate::lib::error::DfxResult;

use crate::lib::installers::assets::content::Content;

pub enum ContentEncoder {
    Gzip,
    Identity,
}

impl ContentEncoder {
    pub fn encode(&self, content: &Content) -> DfxResult<Content> {
        match &self {
            ContentEncoder::Gzip => content.to_gzip(),
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
