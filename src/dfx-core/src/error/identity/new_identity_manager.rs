use crate::error::config::ConfigError;
use crate::error::identity::initialize_identity_manager::InitializeIdentityManagerError;
use crate::error::identity::require_identity_exists::RequireIdentityExistsError;
use crate::error::structured_file::StructuredFileError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NewIdentityManagerError {
    #[error("Failed to get config directory for identity manager")]
    GetConfigDirectoryFailed(#[source] ConfigError),

    #[error("Failed to load identity manager configuration")]
    LoadIdentityManagerConfigurationFailed(#[source] StructuredFileError),

    #[error("Failed to initialize identity manager")]
    InitializeFailed(#[source] InitializeIdentityManagerError),

    #[error("The specified identity must exist")]
    OverrideIdentityMustExist(#[source] RequireIdentityExistsError),

    #[error(r#"No identity configuration found.  Please run "dfx identity get-principal" or "dfx identity new <identity name>" to create a new identity."#)]
    NoIdentityConfigurationFound,
}
