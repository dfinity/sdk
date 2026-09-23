use crate::error::upload_content::UploadContentError;
use ic_agent::AgentError;
use thiserror::Error;

/// Errors related to preparing synchronization operations for a proposal.
#[derive(Error, Debug)]
pub enum PrepareSyncForProposalError {
    /// Failed when querying the asset canister for its API version.
    #[error("Failed to query asset canister API version")]
    ApiVersionQueryFailed(#[source] AgentError),

    /// The asset canister computes evidence with an older encoding than this tool does.
    #[error(
        "The asset canister reports API version {canister_api_version}, but proposing a batch requires API version {required_api_version} or later. Upgrade the asset canister first."
    )]
    EvidenceApiVersionTooLow {
        /// The API version the asset canister reports.
        canister_api_version: u16,
        /// The API version required to compute comparable evidence.
        required_api_version: u16,
    },

    /// Failed while requesting that the asset canister compute evidence.
    #[error("Failed to compute evidence")]
    ComputeEvidence(#[source] AgentError),

    /// Failed while calling propose_commit_batch.
    #[error("Failed to propose batch to commit")]
    ProposeCommitBatch(#[source] AgentError),

    /// Failed while uploading content for synchronization.
    #[error(transparent)]
    UploadContent(#[from] UploadContentError),
}
