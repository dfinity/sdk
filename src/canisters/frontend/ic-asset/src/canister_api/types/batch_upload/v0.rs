use super::common::*;
use candid::{CandidType, Nat};

/// Batch operations that can be applied atomically.
#[derive(CandidType, Debug, Eq, PartialEq, PartialOrd, Ord)]
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
}

/// Apply all of the operations in the batch, and then remove the batch.
#[derive(CandidType, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct CommitBatchArguments {
    /// The batch to commit.
    pub batch_id: Nat,

    /// The operations to apply atomically.
    pub operations: Vec<BatchOperationKind>,
}
