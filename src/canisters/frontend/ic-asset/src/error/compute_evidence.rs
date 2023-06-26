use crate::error::create_project_asset::CreateProjectAssetError;
use crate::error::gather_asset_descriptors::GatherAssetDescriptorsError;
use crate::error::get_asset_properties::GetAssetPropertiesError;
use crate::error::hash_content::HashContentError;

use ic_agent::AgentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ComputeEvidenceError {
    #[error(transparent)]
    ProcessProjectAsset(#[from] CreateProjectAssetError),

    #[error(transparent)]
    GatherAssetDescriptors(#[from] GatherAssetDescriptorsError),

    #[error(transparent)]
    GetAssetProperties(#[from] GetAssetPropertiesError),

    #[error(transparent)]
    HashContent(#[from] HashContentError),

    #[error("Failed to list assets: {0}")]
    ListAssets(AgentError),
}
