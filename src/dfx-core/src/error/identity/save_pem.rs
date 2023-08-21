use crate::error::identity::write_pem_to_file::WritePemToFileError;
use crate::error::keyring::KeyringError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SavePemError {
    #[error("Cannot save PEM content for an HSM.")]
    CannotSavePemContentForHsm(),

    #[error("Failed to write PEM to file: {0}")]
    WritePemToFileFailed(WritePemToFileError),

    #[error("Failed to write PEM to keyring: {0}")]
    WritePemToKeyringFailed(KeyringError),
}
