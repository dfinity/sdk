use super::{config::ConfigError, structured_file::StructuredFileError};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ToolConfigError {
    #[error("Failed to get path for tool configuration")]
    GetConfigPathFailed(#[source] ConfigError),

    #[error("Failed to load tool configuration")]
    LoadConfigFromFileFailed(#[source] StructuredFileError),

    #[error("Failed to save default tool configuration")]
    SaveDefaultConfigFailed(#[source] StructuredFileError),
}
