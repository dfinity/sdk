use crate::asset::content_encoder::ContentEncoder;
use crate::error::create_chunk::CreateChunkError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CreateEncodingError {
    #[error("Failed to create chunk: {0}")]
    CreateChunkFailed(CreateChunkError),

    #[error("Failed to encode content of '{0}' with {1} encoding: {2}")]
    EncodeContentFailed(String, ContentEncoder, std::io::Error),
}
