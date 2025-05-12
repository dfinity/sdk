use super::AssembleCommitBatchArgumentError;
use crate::error::compatibility::CompatibilityError;
use crate::error::create_project_asset::CreateProjectAssetError;
use ic_utils::error::BaseError;
use thiserror::Error;

/// Errors encountered during the upload process.
#[derive(Error, Debug)]
pub enum UploadError {
    /// Failed when calling commit_batch.
    #[error("Commit batch failed")]
    CommitBatchFailed(#[source] BaseError),

    /// Failure when trying to work with an older asset canister.
    #[error(transparent)]
    Compatibility(#[from] CompatibilityError),

    /// Failed when calling create_batch.
    #[error("Create batch failed")]
    CreateBatchFailed(#[source] BaseError),

    /// Failed when assembling commit_batch argument.
    #[error("Failed to assemble commit_batch argument")]
    AssembleCommitBatchArgumentFailed(#[from] AssembleCommitBatchArgumentError),

    /// Failed when creating project assets.
    #[error("Failed to create project asset")]
    CreateProjectAssetFailed(#[from] CreateProjectAssetError),

    /// Failed when calling the list method.
    #[error("List assets failed")]
    ListAssetsFailed(#[source] BaseError),
}
