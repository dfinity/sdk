use thiserror::Error;

#[derive(Error, Debug)]
pub enum GetCurrentExeError {
    #[error("Failed to identify currently running executable: {0}")]
    NoCurrentExe(std::io::Error),
}
