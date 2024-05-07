use crate::error::fs::FsError;
use crate::error::identity::generate_key::GenerateKeyError;
use crate::error::identity::get_legacy_credentials_pem_path::GetLegacyCredentialsPemPathError;
use crate::error::identity::write_pem_to_file::WritePemToFileError;
use crate::error::structured_file::StructuredFileError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InitializeIdentityManagerError {
    #[error("Cannot create identity directory")]
    CreateIdentityDirectoryFailed(#[source] FsError),

    #[error("Failed to generate key")]
    GenerateKeyFailed(#[source] GenerateKeyError),

    #[error(transparent)]
    GetLegacyCredentialsPemPathFailed(#[from] GetLegacyCredentialsPemPathError),

    #[error("Failed to migrate legacy identity")]
    MigrateLegacyIdentityFailed(#[source] FsError),

    #[error("Failed to save configuration")]
    SaveConfigurationFailed(#[source] StructuredFileError),

    #[error("Failed to write pem to file")]
    WritePemToFileFailed(#[source] WritePemToFileError),
}
