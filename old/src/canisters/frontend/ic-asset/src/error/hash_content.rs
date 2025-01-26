use crate::asset::content_encoder::ContentEncoder;
use dfx_core::error::fs::FsError;
use thiserror::Error;

/// Errors related to hashing asset content.
#[derive(Error, Debug)]
pub enum HashContentError {
    /// Failed to encode the content in order to compute the hash.
    #[error("Failed to encode content of '{0}' with {1} encoding: {2}")]
    EncodeContentFailed(String, ContentEncoder, std::io::Error),

    /// Failed to load asset content from the filesystem.
    #[error("Failed to load content: {0}")]
    LoadContentFailed(FsError),
}
