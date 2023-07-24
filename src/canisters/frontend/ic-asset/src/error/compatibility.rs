use crate::error::downgrade_commit_batch_arguments::DowngradeCommitBatchArgumentsV1ToV0Error;
use thiserror::Error;

/// Errors that occur when trying to deploy to an older version of the asset canister.
#[derive(Error, Debug)]
pub enum CompatibilityError {
    /// Failed to downgrade from v1::CommitBatchArguments to v0::CommitBatchArguments.
    #[error("Failed to downgrade from v1::CommitBatchArguments to v0::CommitBatchArguments: {0}. Please upgrade your asset canister, or use older tooling (dfx<=v-0.13.1 or icx-asset<=0.20.0)")]
    DowngradeV1TOV0Failed(DowngradeCommitBatchArgumentsV1ToV0Error),
}
