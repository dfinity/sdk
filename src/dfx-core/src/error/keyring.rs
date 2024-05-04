use crate::error::structured_file::StructuredFileError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KeyringError {
    #[error("Failed to decode pem from keyring")]
    DecodePemFailed(#[source] hex::FromHexError),

    #[error("Failed to delete password from keyring")]
    DeletePasswordFailed(#[source] keyring::Error),

    #[error("Failed to get password for keyring")]
    GetPasswordFailed(#[source] keyring::Error),

    #[error("Failed to load mock keyring")]
    LoadMockKeyringFailed(#[source] StructuredFileError),

    #[error("Mock Keyring: key {0} not found")]
    MockKeyNotFound(String),

    #[error("Mock keyring unavailable - access rejected.")]
    MockUnavailable(),

    #[error("Failed to save mock keyring")]
    SaveMockKeyringFailed(#[source] StructuredFileError),

    #[error("Failed to set password for keyring")]
    SetPasswordFailed(#[source] keyring::Error),
}
