use crate::error::encryption::EncryptionError;
use crate::error::fs::FsError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WritePemToFileError {
    #[error("Failed to encrypt PEM file")]
    EncryptPemFileFailed(PathBuf, #[source] EncryptionError),

    #[error("Failed to write to PEM file")]
    WritePemContentFailed(#[source] FsError),
}
