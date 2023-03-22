use crate::asset::config::HeadersConfig;
use crate::canister_api::types::batch_upload::common::{
    ClearArguments, CreateAssetArguments, DeleteAssetArguments, SetAssetContentArguments,
};
use candid::{CandidType, Nat};
use serde::Deserialize;

/// Batch operations that can be applied atomically.
#[derive(CandidType, Debug)]
#[allow(dead_code)]
pub enum BatchOperationKind {
    /// Create a new asset.
    CreateAsset(CreateAssetArguments),

    /// Assign content to an asset by encoding.
    SetAssetContent(SetAssetContentArguments),

    /// Remove content from an asset by encoding.
    UnsetAssetContent(UnsetAssetContentArguments),

    /// Remove an asset altogether.
    DeleteAsset(DeleteAssetArguments),

    /// Clear all state from the asset canister.
    Clear(ClearArguments),

    /// omment
    SetAssetProperties(SetAssetPropertiesArguments),
}

/// Apply all of the operations in the batch, and then remove the batch.
#[derive(CandidType, Debug)]
pub struct CommitBatchArguments {
    /// The batch to commit.
    pub batch_id: Nat,

    /// The operations to apply atomically.
    pub operations: Vec<BatchOperationKind>,
}
