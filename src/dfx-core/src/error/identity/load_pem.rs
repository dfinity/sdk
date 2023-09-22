use crate::error::identity::load_pem_from_file::LoadPemFromFileError;
use crate::error::keyring::KeyringError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadPemError {
    #[error("Failed to load PEM file from file : {0}")]
    LoadFromFileFailed(LoadPemFromFileError),

    #[error("Failed to load PEM file from keyring for identity '{0}': {1}")]
    LoadFromKeyringFailed(Box<String>, KeyringError),
}
