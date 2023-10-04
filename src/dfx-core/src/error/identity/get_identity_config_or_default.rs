use crate::error::structured_file::StructuredFileError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GetIdentityConfigOrDefaultError {
    #[error("Failed to load configuration for identity '{0}': {1}")]
    LoadIdentityConfigurationFailed(String, StructuredFileError),
}
