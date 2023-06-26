use crate::error::create_project_asset::CreateProjectAssetError;
use crate::error::gather_asset_descriptors::GatherAssetDescriptorsError;
use crate::error::get_asset_properties::GetAssetPropertiesError;

use ic_agent::AgentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UploadContentError {
    #[error("Failed to create batch: {0}")]
    CreateBatchFailed(AgentError),

    #[error("Failed to create project asset: {0}")]
    CreateProjectAssetError(#[from] CreateProjectAssetError),

    #[error("Failed to gather asset descriptors: {0}")]
    GatherAssetDescriptorsFailed(#[from] GatherAssetDescriptorsError),

    #[error(transparent)]
    GetAssetPropertiesFailed(#[from] GetAssetPropertiesError),

    #[error("Failed to list assets: {0}")]
    ListAssetsFailed(AgentError),
}
