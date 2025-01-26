use crate::asset::content_encoder::ContentEncoder;
use crate::error::create_chunk::CreateChunkError;
use thiserror::Error;

/// Errors related to creating/uploading an asset content encoding to the asset canister
#[derive(Error, Debug)]
pub enum CreateEncodingError {
    /// Failed when creating a chunk.
    #[error("Failed to create chunk: {0}")]
    CreateChunkFailed(CreateChunkError),

    /// Failed when encoding asset content.
    #[error("Failed to encode content of '{0}' with {1} encoding: {2}")]
    EncodeContentFailed(String, ContentEncoder, std::io::Error),
}
