use crate::error::config::ConfigError;
use crate::error::identity::initialize_identity_manager::InitializeIdentityManagerError;
use crate::error::identity::IdentityError;
use crate::error::structured_file::StructuredFileError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NewIdentityManagerError {
    #[error("Failed to get config directory for identity manager: {0}")]
    GetConfigDirectoryFailed(ConfigError),

    #[error("Failed to load identity manager configuration: {0}")]
    LoadIdentityManagerConfigurationFailed(StructuredFileError),

    #[error("Failed to initialize identity manager: {0}")]
    InitializeFailed(InitializeIdentityManagerError),

    #[error("The specified identity must exist: {0}")]
    OverrideIdentityMustExist(IdentityError),
}
