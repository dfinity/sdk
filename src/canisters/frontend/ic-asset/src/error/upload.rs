use crate::error::compatibility::CompatibilityError;
use crate::error::create_project_asset::CreateProjectAssetError;

use ic_agent::AgentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UploadError {
    #[error("Commit batch failed: {0}")]
    CommitBatchFailed(AgentError),

    #[error(transparent)]
    Compatibility(#[from] CompatibilityError),

    #[error("Create batch failed: {0}")]
    CreateBatchFailed(AgentError),

    #[error("Failed to create project asset: {0}")]
    CreateProjectAssetFailed(#[from] CreateProjectAssetError),

    #[error("List assets failed: {0}")]
    ListAssetsFailed(AgentError),
}
