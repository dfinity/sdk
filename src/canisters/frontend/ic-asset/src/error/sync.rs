use crate::error::compatibility::CompatibilityError;
use crate::error::upload_content::UploadContentError;
use ic_utils::error::BaseError;
use thiserror::Error;

/// Errors related to the sync process.
#[derive(Error, Debug)]
pub enum SyncError {
    /// Failed when calling commit_batch
    #[error("Failed to commit batch")]
    CommitBatchFailed(#[source] BaseError),

    /// Failed when trying to work with an older asset canister.
    #[error(transparent)]
    Compatibility(#[from] CompatibilityError),

    /// Failed when uploading content for synchronization.
    #[error(transparent)]
    UploadContentFailed(#[from] UploadContentError),
}
