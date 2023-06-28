use thiserror::Error;

#[derive(Error, Debug)]
pub enum ArchiveError {
    #[error("Failed to read archive path: {0}")]
    ArchiveFileInvalidPath(std::io::Error),
}
