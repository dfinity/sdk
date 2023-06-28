use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessError {
    #[error("Execution of '{0:?}' failed: {1}")]
    ExecutionFailed(std::ffi::OsString, std::io::Error),
}
