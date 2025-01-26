use thiserror::Error;

/// Errors related to downgrading CommitBatchArguments for use with a v0 asset canister.
#[derive(Error, Debug)]
pub enum DowngradeCommitBatchArgumentsV1ToV0Error {
    /// Asset canister v0 does not support SetAssetProperties.
    #[error("SetAssetProperties is not supported")]
    V0SetAssetPropertiesNotSupported,
}
