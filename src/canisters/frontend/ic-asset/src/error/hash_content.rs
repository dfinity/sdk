use crate::asset::content_encoder::ContentEncoder;

use dfx_core::error::fs::FsError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HashContentError {
    #[error("Failed to encode content of '{0}' with {1} encoding: {2}")]
    EncodeContentFailed(String, ContentEncoder, std::io::Error),

    #[error("Failed to load content: {0}")]
    LoadContentFailed(FsError),
}
