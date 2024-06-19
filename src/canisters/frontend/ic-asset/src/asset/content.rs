use crate::asset::content_encoder::ContentEncoder;
use brotli::CompressorWriter;
use dfx_core::error::fs::FsError;
use flate2::write::GzEncoder;
use flate2::Compression;
use mime::Mime;
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::Path;

#[derive(Clone)]
pub(crate) struct Content {
    pub data: Vec<u8>,
    pub media_type: Mime,
}

impl Content {
    pub fn load(path: &Path) -> Result<Content, FsError> {
        let data = dfx_core::fs::read(path)?;

        // todo: check contents if mime_guess fails https://github.com/dfinity/sdk/issues/1594
        let media_type = mime_guess::from_path(path)
            .first()
            .unwrap_or(mime::APPLICATION_OCTET_STREAM);

        Ok(Content { data, media_type })
    }

    pub fn encode(&self, encoder: &ContentEncoder) -> Result<Content, std::io::Error> {
        match encoder {
            ContentEncoder::Gzip => self.to_gzip(),
            ContentEncoder::Brotli => self.to_brotli(),
            ContentEncoder::Identity => Ok(self.clone()),
        }
    }

    pub fn to_gzip(&self) -> Result<Content, std::io::Error> {
        let mut e = GzEncoder::new(Vec::new(), Compression::default());
        e.write_all(&self.data)?;
        let data = e.finish()?;
        Ok(Content {
            data,
            media_type: self.media_type.clone(),
        })
    }

    pub fn to_brotli(&self) -> Result<Content, std::io::Error> {
        let mut compressed_data = Vec::new();
        {
            let mut compressor = CompressorWriter::new(&mut compressed_data, 4096, 11, 22);
            compressor.write_all(&self.data)?;
            compressor.flush()?;
        }
        Ok(Content {
            data: compressed_data,
            media_type: self.media_type.clone(),
        })
    }

    pub fn sha256(&self) -> Vec<u8> {
        Sha256::digest(&self.data).to_vec()
    }
}
