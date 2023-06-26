use ic_agent::AgentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CreateChunkError {
    #[error(transparent)]
    Agent(#[from] AgentError),

    #[error("Failed to decode create chunk response: {0}")]
    DecodeCreateChunkResponseFailed(candid::Error),
}
