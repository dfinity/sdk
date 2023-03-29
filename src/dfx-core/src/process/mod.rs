use crate::error::process::ProcessError;
use std::process::{Command, ExitStatus};

pub fn execute_process(cmd: &mut Command) -> Result<ExitStatus, ProcessError> {
    cmd.status()
        .map_err(|e| ProcessError::ExecutionFailed(cmd.get_program().to_owned(), e))
}
