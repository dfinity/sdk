use crate::error::encryption::EncryptionError;
use crate::error::fs::FsError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadPemFromFileError {
    #[error("Failed to decrypt PEM file at {0}")]
    DecryptPemFileFailed(PathBuf, #[source] EncryptionError),

    #[error("Failed to read pem file")]
    ReadPemFileFailed(#[source] FsError),
}
