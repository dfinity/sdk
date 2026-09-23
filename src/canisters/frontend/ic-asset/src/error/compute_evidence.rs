use crate::error::create_project_asset::CreateProjectAssetError;
use crate::error::gather_asset_descriptors::GatherAssetDescriptorsError;
use crate::error::get_asset_properties::GetAssetPropertiesError;
use crate::error::hash_content::HashContentError;
use ic_agent::AgentError;
use thiserror::Error;

use super::AssembleCommitBatchArgumentError;

/// Errors related to computing evidence for a proposed update.
#[derive(Error, Debug)]
pub enum ComputeEvidenceError {
    /// Failed when querying the asset canister for its API version.
    #[error("Failed to query asset canister API version")]
    ApiVersionQueryFailed(#[source] AgentError),

    /// The asset canister computes evidence with an older encoding than this tool does.
    #[error(
        "The asset canister reports API version {canister_api_version}, but computing evidence to compare with it requires API version {required_api_version} or later. Upgrade the asset canister first."
    )]
    EvidenceApiVersionTooLow {
        /// The API version the asset canister reports.
        canister_api_version: u16,
        /// The API version required to compute comparable evidence.
        required_api_version: u16,
    },

    /// Failed when assembling commit_batch argument.
    #[error(transparent)]
    AssembleCommitBatchArgumentFailed(#[from] AssembleCommitBatchArgumentError),

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
    #[error("Failed to list assets")]
    ListAssets(#[source] AgentError),
}
