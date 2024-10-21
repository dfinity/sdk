use thiserror::Error;

use super::SetEncodingError;

/// Errors related to assembling commit_batch arguments for the asset canister.
#[derive(Error, Debug)]
pub enum AssembleCommitBatchArgumentError {
    /// Failed to set encoding.
    #[error("Failed to set encoding: {0}")]
    SetEncodingFailed(#[from] SetEncodingError),
}
