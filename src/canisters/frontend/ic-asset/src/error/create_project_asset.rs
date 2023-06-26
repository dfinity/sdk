use crate::error::create_encoding::CreateEncodingError;
use dfx_core::error::fs::FsError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CreateProjectAssetError {
    #[error("Failed to create encoding: {0}")]
    CreateEncodingError(#[from] CreateEncodingError),

    #[error("Failed to determine asset size: {0}")]
    DetermineAssetSizeFailed(FsError),

    #[error("Failed to load asset content: {0}")]
    LoadContentFailed(FsError),
}
