use crate::asset::config::HeadersConfig;
use candid::{CandidType, Nat};
use serde::Deserialize;

/// Create a new batch, which will expire after some time period.
/// This expiry is extended by any call to create_chunk().
/// Also, removes any expired batches.
#[derive(CandidType, Debug)]
pub struct CreateBatchRequest {}

/// The response to a CreateBatchRequest.
#[derive(CandidType, Debug, Deserialize)]
pub struct CreateBatchResponse {
    /// The ID of the created batch.
    pub batch_id: Nat,
}

/// Upload a chunk of data that is part of an asset's content.
#[derive(CandidType, Debug, Deserialize)]
pub struct CreateChunkRequest<'a> {
    /// The batch with which to associate the created chunk.
    /// The chunk will be deleted if the batch expires before being committed.
    pub batch_id: Nat,

    /// The data in this chunk.
    #[serde(with = "serde_bytes")]
    pub content: &'a [u8],
}

/// The response to a CreateChunkRequest.
#[derive(CandidType, Debug, Deserialize)]
pub struct CreateChunkResponse {
    /// The ID of the created chunk.
    pub chunk_id: Nat,
}

/// Upload multiple chunks of data that are part of asset content.
#[derive(CandidType, Debug, Deserialize)]
pub struct CreateChunksRequest {
    /// The batch with which to associate the created chunks.
    /// The chunk will be deleted if the batch expires before being committed.
    pub batch_id: Nat,

    /// The data in this chunk.
    pub content: Vec<Vec<u8>>,
}

/// The response to a CreateChunksRequest.
#[derive(CandidType, Debug, Deserialize)]
pub struct CreateChunksResponse {
    /// The IDs of the created chunks.
    pub chunk_ids: Vec<Nat>,
}

/// Create a new asset.  Has no effect if the asset already exists and the content type matches.
/// Traps if the asset already exists but with a different content type.
#[derive(CandidType, Clone, Debug, PartialOrd, PartialEq, Eq, Ord)]
pub struct CreateAssetArguments {
    /// The key identifies the asset.
    pub key: String,
    /// The MIME type of this asset
    pub content_type: String,
    /// The cache HTTP header Time To Live parameter
    pub max_age: Option<u64>,
    /// The HTTP headers
    pub headers: Option<HeadersConfig>,
    /// Aliasing enabled or not
    pub enable_aliasing: Option<bool>,
    /// When set to true, don't redirect from raw to certified
    pub allow_raw_access: Option<bool>,
}

/// Set the data for a particular content encoding for the given asset.
#[derive(CandidType, Clone, Debug, PartialOrd, PartialEq, Eq, Ord)]
pub struct SetAssetContentArguments {
    /// The key identifies the asset.
    pub key: String,
    /// The content encoding for which this content applies
    pub content_encoding: String,
    /// The chunks to assign to this content
    pub chunk_ids: Vec<Nat>,
    /// If no chunks are assigned to this asset, then `asset_content` is the asset content
    pub asset_content: Option<Vec<u8>>,
    /// The sha256 of the entire content
    pub sha256: Option<Vec<u8>>,
}

/// Remove a specific content encoding for the asset.
#[derive(CandidType, Clone, Debug, PartialOrd, PartialEq, Eq, Ord)]
pub struct UnsetAssetContentArguments {
    /// The key identifies the asset.
    pub key: String,
    /// The content encoding to remove.
    pub content_encoding: String,
}

/// Remove the specified asset.
#[derive(CandidType, Clone, Debug, PartialOrd, PartialEq, Eq, Ord)]
pub struct DeleteAssetArguments {
    /// The key identifies the asset to delete.
    pub key: String,
}

/// Remove all assets, batches, and chunks, and reset the next batch and chunk IDs.
#[derive(CandidType, Clone, Debug, PartialOrd, PartialEq, Eq, Ord)]
pub struct ClearArguments {}

/// Compute a hash over the proposed CommitBatchArguments.  This may take more than one call.
#[derive(CandidType, Debug)]
pub struct ComputeEvidenceArguments {
    /// The batch for which to compute evidence
    pub batch_id: Nat,

    /// A measure of how much work to do in one call
    pub max_iterations: Option<u16>,
}
