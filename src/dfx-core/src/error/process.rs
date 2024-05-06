use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessError {
    #[error("Execution of '{0:?}' failed")]
    ExecutionFailed(std::ffi::OsString, #[source] std::io::Error),
}
