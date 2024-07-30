use crate::error::create_encoding::CreateEncodingError;
use dfx_core::error::fs::{ReadFileError, ReadMetadataError};
use thiserror::Error;

/// Errors related to creating an asset found in the project in the asset canister.
#[derive(Error, Debug)]
pub enum CreateProjectAssetError {
    /// Failed to create an asset encoding in the asset canister.
    #[error("Failed to create encoding: {0}")]
    CreateEncodingError(#[from] CreateEncodingError),

    /// Failed to find out the file size of an asset.
    #[error("failed to determine asset size")]
    DetermineAssetSizeFailed(#[from] ReadMetadataError),

    /// Failed to load asset content from the filesystem.
    #[error("failed to load asset content")]
    LoadContentFailed(#[from] ReadFileError),
}
