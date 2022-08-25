use crate::content_encoder::ContentEncoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use mime::Mime;
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::Path;

pub(crate) struct Content {
    pub data: Vec<u8>,
    pub media_type: Mime,
}

impl Content {
    pub fn load(path: &Path) -> anyhow::Result<Content> {
        let data = std::fs::read(path)?;

        // todo: check contents if mime_guess fails https://github.com/dfinity/sdk/issues/1594
        let media_type = mime_guess::from_path(path)
            .first()
            .unwrap_or(mime::APPLICATION_OCTET_STREAM);

        Ok(Content { data, media_type })
    }

    pub fn encode(&self, encoder: &ContentEncoder) -> anyhow::Result<Content> {
        match encoder {
            ContentEncoder::Gzip => self.to_gzip(),
        }
    }

    pub fn to_gzip(&self) -> anyhow::Result<Content> {
        let mut e = GzEncoder::new(Vec::new(), Compression::default());
        e.write_all(&self.data)?;
        let data = e.finish()?;
        Ok(Content {
            data,
            media_type: self.media_type.clone(),
        })
    }

    pub fn sha256(&self) -> Vec<u8> {
        Sha256::digest(&self.data).to_vec()
    }
}
