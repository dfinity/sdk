use crate::error::identity::get_identity_config_or_default::GetIdentityConfigOrDefaultError;
use crate::error::identity::load_pem::LoadPemError;
use crate::error::identity::require_identity_exists::RequireIdentityExistsError;
use crate::error::identity::validate_pem_file::ValidatePemFileError;
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExportIdentityError {
    #[error("Failed to get identity config")]
    GetIdentityConfigFailed(#[source] GetIdentityConfigOrDefaultError),

    #[error("The specified identity does not exist")]
    IdentityDoesNotExist(#[source] RequireIdentityExistsError),

    #[error("Failed to load pem file")]
    LoadPemFailed(#[source] LoadPemError),

    #[error("Could not translate pem file to text")]
    TranslatePemContentToTextFailed(#[source] FromUtf8Error),

    #[error("Failed to validate pem file")]
    ValidatePemFileFailed(#[source] ValidatePemFileError),
}
