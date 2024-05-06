use crate::error::fs::FsError;
use crate::error::structured_file::StructuredFileError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SaveIdentityConfigurationError {
    #[error("Failed to ensure identity configuration directory exists")]
    EnsureIdentityConfigurationDirExistsFailed(#[source] FsError),

    #[error("Failed to save identity configuration")]
    SaveIdentityConfigurationFailed(#[source] StructuredFileError),
}
