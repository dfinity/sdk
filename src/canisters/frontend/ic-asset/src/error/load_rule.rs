use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadRuleError {
    #[error("Failed to combine {0} and {1} into a string (to be later used as a glob pattern)")]
    FormGlobPatternFailed(PathBuf, String),

    #[error("{0} is not a valid glob pattern: {1}")]
    InvalidGlobPattern(String, globset::Error),
}
