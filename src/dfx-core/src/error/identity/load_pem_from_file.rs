use crate::error::encryption::EncryptionError;
use crate::error::fs::FsError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadPemFromFileError {
    #[error("Failed to decrypt PEM file: {0}")]
    DecryptPemFileFailed(PathBuf, EncryptionError),

    #[error("Failed to read pem file: {0}")]
    ReadPemFileFailed(FsError),
}
