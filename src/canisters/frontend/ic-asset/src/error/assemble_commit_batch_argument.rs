use thiserror::Error;

use super::SetEncodingError;

/// Errors related to creating an asset found in the project in the asset canister.
#[derive(Error, Debug)]
pub enum AssembleCommitBatchArgumentError {
    /// Failed to set encoding.
    #[error("Failed to set encoding: {0}")]
    SetEncodingFailed(#[from] SetEncodingError),
}
