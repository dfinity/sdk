use crate::error::config::ApplyExtensionCanisterTypesError;
use crate::error::fs::FsError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadDfxConfigError {
    #[error(transparent)]
    ApplyExtensionCanisterTypesError(#[from] ApplyExtensionCanisterTypesError),

    #[error("Failed to deserialize json from {0}: {1}")]
    DeserializeValueFailed(Box<PathBuf>, serde_json::Error),

    #[error("Failed to resolve config path: {0}")]
    ResolveConfigPathFailed(FsError),

    #[error("Failed to load dfx configuration: {0}")]
    ReadFile(FsError),

    #[error("Failed to determine current working dir: {0}")]
    DetermineCurrentWorkingDirFailed(std::io::Error),
}
