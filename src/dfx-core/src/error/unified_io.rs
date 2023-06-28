use crate::error::archive::ArchiveError;
use crate::error::fs::FsError;
use crate::error::structured_file::StructuredFileError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum UnifiedIoError {
    #[error(transparent)]
    FsError(#[from] FsError),

    #[error(transparent)]
    StructuredFile(#[from] StructuredFileError),

    #[error(transparent)]
    Archive(#[from] ArchiveError),
}
