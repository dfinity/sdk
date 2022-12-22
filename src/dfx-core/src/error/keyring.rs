use crate::error::structured_file::StructuredFileError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum KeyringError {
    #[error("Failed to delete password from keyring: {0}")]
    DeletePasswordFailed(keyring::Error),

    #[error("Failed to load mock keyring: {0}")]
    LoadMockKeyringFailed(StructuredFileError),

    #[error("Mock keyring unavailable - access rejected.")]
    MockUnavailable(),

    #[error("Failed to save mock keyring: {0}")]
    SaveMockKeyringFailed(StructuredFileError),
}
