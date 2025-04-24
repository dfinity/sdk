use ic_utils::error::BaseError;
use thiserror::Error;

/// Errors related to creating a chunk.
#[derive(Error, Debug)]
pub enum CreateChunkError {
    /// Failed in call to create_chunk, or in waiting for response.
    #[error("Failed to create chunk")]
    CreateChunk(#[source] BaseError),

    /// Failed in call to create_chunks, or in waiting for response.
    #[error("Failed to create chunks")]
    CreateChunks(#[source] BaseError),

    /// Failed to decode the create chunk response.
    #[error("Failed to decode create chunk response")]
    DecodeCreateChunkResponse(#[source] candid::Error),
}
