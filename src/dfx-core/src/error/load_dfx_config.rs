use crate::error::config::ApplyExtensionCanisterTypesError;
use crate::error::fs::FsError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadDfxConfigError {
    #[error(transparent)]
    ApplyExtensionCanisterTypesError(#[from] ApplyExtensionCanisterTypesError),

    #[error("Failed to deserialize json from {0}")]
    DeserializeValueFailed(Box<PathBuf>, #[source] serde_json::Error),

    #[error("Failed to resolve config path")]
    ResolveConfigPathFailed(#[source] FsError),

    #[error("Failed to load dfx configuration")]
    ReadFile(#[source] FsError),

    #[error("Failed to determine current working dir")]
    DetermineCurrentWorkingDirFailed(#[source] std::io::Error),
}
