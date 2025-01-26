use thiserror::Error;

#[derive(Error, Debug)]
pub enum GetCurrentExeError {
    #[error("Failed to identify currently running executable")]
    NoCurrentExe(#[source] std::io::Error),
}
