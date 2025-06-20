use thiserror::Error;

#[derive(Error, Debug)]
#[error("Failed to read archive path")]
pub struct GetArchivePathError(#[source] pub std::io::Error);
