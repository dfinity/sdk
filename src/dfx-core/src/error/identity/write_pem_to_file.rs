use crate::error::encryption::EncryptionError;
use crate::error::fs::FsError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WritePemToFileError {
    #[error("Failed to encrypt PEM file: {0}")]
    EncryptPemFileFailed(PathBuf, EncryptionError),

    #[error("Failed to write to PEM file: {0}")]
    WritePemContentFailed(FsError),
}
