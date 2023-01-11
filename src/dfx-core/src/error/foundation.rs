use thiserror::Error;

#[derive(Error, Debug)]
pub enum FoundationError {
    #[error("Cannot find home directory (no HOME environment variable).")]
    NoHomeInEnvironment(),
}
