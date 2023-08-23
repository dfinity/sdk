use crate::error::fs::FsError;
use crate::error::structured_file::StructuredFileError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadDfxConfigError {
    #[error("Failed to resolve config path: {0}")]
    ResolveConfigPathFailed(FsError),

    #[error("Failed to load dfx configuration: {0}")]
    LoadFromFileFailed(StructuredFileError),

    #[error("Failed to determine current working dir: {0}")]
    DetermineCurrentWorkingDirFailed(std::io::Error),
}
