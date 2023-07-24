use crate::error::structured_file::StructuredFileError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KeyringError {
    #[error("Failed to decode pem from keyring: {0}")]
    DecodePemFailed(hex::FromHexError),

    #[error("Failed to delete password from keyring: {0}")]
    DeletePasswordFailed(keyring::Error),

    #[error("Failed to get password for keyring: {0}")]
    GetPasswordFailed(keyring::Error),

    #[error("Failed to load mock keyring: {0}")]
    LoadMockKeyringFailed(StructuredFileError),

    #[error("Mock Keyring: key {0} not found")]
    MockKeyNotFound(String),

    #[error("Mock keyring unavailable - access rejected.")]
    MockUnavailable(),

    #[error("Failed to save mock keyring: {0}")]
    SaveMockKeyringFailed(StructuredFileError),

    #[error("Failed to set password for keyring: {0}")]
    SetPasswordFailed(keyring::Error),
}
