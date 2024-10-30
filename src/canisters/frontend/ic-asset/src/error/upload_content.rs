use crate::error::create_project_asset::CreateProjectAssetError;
use crate::error::gather_asset_descriptors::GatherAssetDescriptorsError;
use crate::error::get_asset_properties::GetAssetPropertiesError;
use ic_agent::AgentError;
use thiserror::Error;

use super::AssembleCommitBatchArgumentError;

/// Errors related to uploading content to the asset canister.
#[derive(Error, Debug)]
pub enum UploadContentError {
    /// Failed when assembling commit_batch argument.
    #[error("Failed to assemble commit_batch argument: {0}")]
    AssembleCommitBatchArgumentFailed(AssembleCommitBatchArgumentError),

    /// Failed when calling create_batch.
    #[error("Failed to create batch: {0}")]
    CreateBatchFailed(AgentError),

    /// Failed when creating project assets.
    #[error("Failed to create project asset: {0}")]
    CreateProjectAssetError(#[from] CreateProjectAssetError),

    /// Failed when building list of assets to synchronize.
    #[error("Failed to gather asset descriptors: {0}")]
    GatherAssetDescriptorsFailed(#[from] GatherAssetDescriptorsError),

    /// Failed when getting asset properties.
    #[error(transparent)]
    GetAssetPropertiesFailed(#[from] GetAssetPropertiesError),

    /// Failed when calling the list method.
    #[error("Failed to list assets: {0}")]
    ListAssetsFailed(AgentError),
}
