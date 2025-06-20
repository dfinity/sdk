use crate::error::config::ApplyExtensionCanisterTypesError;
use crate::error::fs::{CanonicalizePathError, ReadFileError};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadDfxConfigError {
    #[error(transparent)]
    ApplyExtensionCanisterTypesError(#[from] ApplyExtensionCanisterTypesError),

    #[error("Failed to deserialize json from {0}")]
    DeserializeValueFailed(Box<PathBuf>, #[source] serde_json::Error),

    #[error("failed to resolve config path")]
    ResolveConfigPath(#[source] CanonicalizePathError),

    #[error("Failed to load dfx configuration")]
    ReadFile(#[from] ReadFileError),

    #[error("Failed to determine current working dir")]
    DetermineCurrentWorkingDirFailed(#[source] std::io::Error),
}
