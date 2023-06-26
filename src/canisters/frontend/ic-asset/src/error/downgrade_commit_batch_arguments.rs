use thiserror::Error;

#[derive(Error, Debug)]
pub enum DowngradeCommitBatchArgumentsV1ToV0Error {
    #[error("SetAssetProperties is not supported")]
    V0SetAssetPropertiesNotSupported,
}
