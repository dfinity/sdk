use crate::error::compatibility::CompatibilityError;
use crate::error::create_project_asset::CreateProjectAssetError;
use ic_agent::AgentError;
use thiserror::Error;

/// Errors encountered during the upload process.
#[derive(Error, Debug)]
pub enum UploadError {
    /// Failed when calling commit_batch.
    #[error("Commit batch failed: {0}")]
    CommitBatchFailed(AgentError),

    /// Failure when trying to work with an older asset canister.
    #[error(transparent)]
    Compatibility(#[from] CompatibilityError),

    /// Failed when calling create_batch.
    #[error("Create batch failed: {0}")]
    CreateBatchFailed(AgentError),

    /// Failed when assembling commit_batch argument.
    #[error("Failed to assemble commit_batch argument: {0}")]
    AssembleCommitBatchArgumentError(String),

    /// Failed when creating project assets.
    #[error("Failed to create project asset: {0}")]
    CreateProjectAssetFailed(#[from] CreateProjectAssetError),

    /// Failed when calling the list method.
    #[error("List assets failed: {0}")]
    ListAssetsFailed(AgentError),
}
