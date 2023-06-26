use crate::error::upload_content::UploadContentError;

use ic_agent::AgentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PrepareSyncForProposalError {
    #[error("Failed to compute evidence: {0}")]
    ComputeEvidence(AgentError),

    #[error("Failed to propose batch to commit: {0}")]
    ProposeCommitBatch(AgentError),

    #[error(transparent)]
    UploadContent(#[from] UploadContentError),
}
