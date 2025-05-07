use crate::error::upload_content::UploadContentError;
use ic_utils::error::BaseError;
use thiserror::Error;

/// Errors related to preparing synchronization operations for a proposal.
#[derive(Error, Debug)]
pub enum PrepareSyncForProposalError {
    /// Failed while requesting that the asset canister compute evidence.
    #[error("Failed to compute evidence")]
    ComputeEvidence(#[source] BaseError),

    /// Failed while calling propose_commit_batch.
    #[error("Failed to propose batch to commit")]
    ProposeCommitBatch(#[source] BaseError),

    /// Failed while uploading content for synchronization.
    #[error(transparent)]
    UploadContent(#[from] UploadContentError),
}
