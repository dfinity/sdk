use thiserror::Error;

#[derive(Debug, Error)]
#[error("CLI error: {0}")]
pub struct CliError(pub String);

pub type CliResult = Result<(), CliError>;
