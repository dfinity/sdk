use crate::error::create_encoding::CreateEncodingError;
use dfx_core::error::fs::FsError;
use thiserror::Error;

/// Errors related to creating an asset found in the project in the asset canister.
#[derive(Error, Debug)]
pub enum CreateProjectAssetError {
    /// Failed to create an asset encoding in the asset canister.
    #[error("Failed to create encoding: {0}")]
    CreateEncodingError(#[from] CreateEncodingError),

    /// Failed to find out the file size of an asset.
    #[error("Failed to determine asset size: {0}")]
    DetermineAssetSizeFailed(FsError),

    /// Failed to load asset content from the filesystem.
    #[error("Failed to load asset content: {0}")]
    LoadContentFailed(FsError),
}
