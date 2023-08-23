use crate::canister_api::types::{
    asset::SetAssetPropertiesArguments,
    batch_upload::common::{
        ClearArguments, CreateAssetArguments, DeleteAssetArguments, SetAssetContentArguments,
        UnsetAssetContentArguments,
    },
};
use candid::{CandidType, Nat};
use std::collections::HashMap;

/// Batch operations that can be applied atomically.
#[derive(CandidType, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum BatchOperationKind {
    #[allow(dead_code)]
    /// Clear all state from the asset canister.
    Clear(ClearArguments),

    /// Remove an asset altogether.
    DeleteAsset(DeleteAssetArguments),

    /// Create a new asset.
    CreateAsset(CreateAssetArguments),

    /// Remove content from an asset by encoding.
    UnsetAssetContent(UnsetAssetContentArguments),

    /// Assign content to an asset by encoding.
    SetAssetContent(SetAssetContentArguments),

    /// Set asset properties.
    SetAssetProperties(SetAssetPropertiesArguments),
}

/// Apply all of the operations in the batch, and then remove the batch.
#[derive(CandidType, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct CommitBatchArguments {
    /// The batch to commit.
    pub batch_id: Nat,

    /// The operations to apply atomically.
    pub operations: Vec<BatchOperationKind>,
}

impl CommitBatchArguments {
    pub(crate) fn group_by_kind_then_count(&self) -> HashMap<String, usize> {
        self.operations
            .iter()
            .fold(HashMap::new(), |mut map: HashMap<String, usize>, op| {
                let key = match op {
                    BatchOperationKind::Clear(_) => "Clear",
                    BatchOperationKind::DeleteAsset(_) => "Delete",
                    BatchOperationKind::CreateAsset(_) => "CreateAsset",
                    BatchOperationKind::UnsetAssetContent(_) => "UnsetAssetContent",
                    BatchOperationKind::SetAssetContent(_) => "SetAssetContent",
                    BatchOperationKind::SetAssetProperties(_) => "SetAssetProperties",
                };
                *map.entry(key.to_owned()).or_default() += 1;
                map
            })
    }
}
