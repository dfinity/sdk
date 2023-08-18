use crate::error::fs::FsError;
use crate::error::structured_file::StructuredFileError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SaveIdentityConfigurationError {
    #[error("Failed to ensure identity configuration directory exists: {0}")]
    EnsureIdentityConfigurationDirExistsFailed(FsError),

    #[error("Failed to save identity configuration: {0}")]
    SaveIdentityConfigurationFailed(StructuredFileError),
}
