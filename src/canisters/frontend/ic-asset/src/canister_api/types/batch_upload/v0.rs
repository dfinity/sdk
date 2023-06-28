use super::common::*;
use crate::error::DowngradeCommitBatchArgumentsV1ToV0Error;
use crate::error::DowngradeCommitBatchArgumentsV1ToV0Error::V0SetAssetPropertiesNotSupported;

use candid::{CandidType, Nat};

/// Batch operations that can be applied atomically.
#[derive(CandidType, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[allow(dead_code)]
pub enum BatchOperationKind {
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
}

/// Apply all of the operations in the batch, and then remove the batch.
#[derive(CandidType, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct CommitBatchArguments {
    /// The batch to commit.
    pub batch_id: Nat,

    /// The operations to apply atomically.
    pub operations: Vec<BatchOperationKind>,
}

// impl try_from for v0::BatchOperationKind from v1::BatchOperationKind
impl TryFrom<super::v1::CommitBatchArguments> for CommitBatchArguments {
    type Error = DowngradeCommitBatchArgumentsV1ToV0Error;

    fn try_from(value: super::v1::CommitBatchArguments) -> Result<Self, Self::Error> {
        let mut operations = vec![];
        for operation in value.operations {
            let operation = match operation {
                super::v1::BatchOperationKind::CreateAsset(args) => {
                    BatchOperationKind::CreateAsset(args)
                }
                super::v1::BatchOperationKind::SetAssetContent(args) => {
                    BatchOperationKind::SetAssetContent(args)
                }
                super::v1::BatchOperationKind::UnsetAssetContent(args) => {
                    BatchOperationKind::UnsetAssetContent(args)
                }
                super::v1::BatchOperationKind::DeleteAsset(args) => {
                    BatchOperationKind::DeleteAsset(args)
                }
                super::v1::BatchOperationKind::Clear(args) => BatchOperationKind::Clear(args),
                super::v1::BatchOperationKind::SetAssetProperties(_) => {
                    return Err(V0SetAssetPropertiesNotSupported)
                }
            };
            operations.push(operation);
        }

        Ok(Self {
            batch_id: value.batch_id,
            operations,
        })
    }
}
