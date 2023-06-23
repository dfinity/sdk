use crate::error::compatibility::CompatibilityError;
use crate::error::upload_content::UploadContentError;

use ic_agent::AgentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SyncError {
    #[error("Failed to commit batch: {0}")]
    CommitBatchFailed(AgentError),

    #[error(transparent)]
    Compatibility(#[from] CompatibilityError),

    #[error(transparent)]
    UploadContentFailed(#[from] UploadContentError),
}
