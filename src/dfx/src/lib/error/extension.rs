use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExtensionError {
    #[error("Cannot create cache directory at '{0}'.")]
    CreateExtensionDirectoryFailed(PathBuf),

    #[error("Cannot find cache directory at '{0}'.")]
    FindExtensionDirectoryFailed(PathBuf),

    // Windows paths do not require environment variables (and are found by dirs-next, which has its own errors)
    #[cfg(not(windows))]
    #[error("Cannot find home directory.")]
    NoHomeInEnvironment(),

    #[error("Unknown version '{0}'.")]
    UnknownVersion(String),

    #[error("Generic extension error '{0}'.")]
    ExtensionError(String),
}
