use crate::error::create_project_asset::CreateProjectAssetError;
use crate::error::gather_asset_descriptors::GatherAssetDescriptorsError;
use crate::error::get_asset_properties::GetAssetPropertiesError;
use crate::error::hash_content::HashContentError;

use ic_agent::AgentError;
use thiserror::Error;

/// Errors related to computing evidence for a proposed update.
#[derive(Error, Debug)]
pub enum ComputeEvidenceError {
    /// Failed when inspecting assets to be updated.
    #[error(transparent)]
    ProcessProjectAsset(#[from] CreateProjectAssetError),

    /// Failed when determining which assets and encodings changed.
    #[error(transparent)]
    GatherAssetDescriptors(#[from] GatherAssetDescriptorsError),

    /// Failed when reading assets properties from the asset canister.
    #[error(transparent)]
    GetAssetProperties(#[from] GetAssetPropertiesError),

    /// Failed when computing hashes of asset content.
    #[error(transparent)]
    HashContent(#[from] HashContentError),

    /// Failed to list assets in the asset canister.
    #[error("Failed to list assets: {0}")]
    ListAssets(AgentError),
}
