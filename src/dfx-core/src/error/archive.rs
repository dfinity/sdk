use thiserror::Error;

#[derive(Error, Debug)]
pub enum ArchiveError {
    #[error("Failed to read archive path")]
    ArchiveFileInvalidPath(#[source] std::io::Error),
}
