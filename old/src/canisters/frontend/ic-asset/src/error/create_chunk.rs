use ic_agent::AgentError;
use thiserror::Error;

/// Errors related to creating a chunk.
#[derive(Error, Debug)]
pub enum CreateChunkError {
    /// Failed in call to create_chunk, or in waiting for response.
    #[error("Failed to create chunk: {0}")]
    CreateChunk(AgentError),

    /// Failed to decode the create chunk response.
    #[error("Failed to decode create chunk response: {0}")]
    DecodeCreateChunkResponse(candid::Error),
}
