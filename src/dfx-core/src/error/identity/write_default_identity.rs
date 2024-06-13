use crate::error::structured_file::StructuredFileError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WriteDefaultIdentityError {
    #[error("Failed to save identity manager configuration")]
    SaveIdentityManagerConfigurationFailed(#[source] StructuredFileError),
}
