use crate::error::identity::load_pem_from_file::LoadPemFromFileError;
use crate::error::keyring::KeyringError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadPemError {
    #[error("Failed to load PEM file from file")]
    LoadFromFileFailed(#[source] LoadPemFromFileError),

    #[error("Failed to load PEM file from keyring for identity '{0}'")]
    LoadFromKeyringFailed(Box<String>, #[source] KeyringError),
}
