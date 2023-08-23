use crate::error::fs::FsError;
use crate::error::identity::generate_key::GenerateKeyError;
use crate::error::identity::get_legacy_credentials_pem_path::GetLegacyCredentialsPemPathError;
use crate::error::identity::write_pem_to_file::WritePemToFileError;
use crate::error::identity::IdentityError;
use crate::error::structured_file::StructuredFileError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InitializeIdentityManagerError {
    #[error("Cannot create identity directory: {0}")]
    CreateIdentityDirectoryFailed(FsError),

    #[error("Failed to generate key: {0}")]
    GenerateKeyFailed(GenerateKeyError),

    #[error(transparent)]
    GetLegacyCredentialsPemPathFailed(#[from] GetLegacyCredentialsPemPathError),

    #[error("Failed to migrate legacy identity")]
    MigrateLegacyIdentityFailed(FsError),

    #[error("Failed to save configuration: {0}")]
    SaveConfigurationFailed(StructuredFileError),

    #[error("Failed to write pem to file: {0}")]
    WritePemToFileFailed(WritePemToFileError),
}
