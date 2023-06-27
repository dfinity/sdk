use std::path::PathBuf;
use thiserror::Error;

/// Errors related to loading an asset configuration rule.
#[derive(Error, Debug)]
pub enum LoadRuleError {
    /// The match string could not be combined with the root directory to form a valid string.
    #[error("Failed to combine {0} and {1} into a string (to be later used as a glob pattern)")]
    FormGlobPatternFailed(PathBuf, String),

    /// The glob pattern was not valid.
    #[error("{0} is not a valid glob pattern: {1}")]
    InvalidGlobPattern(String, globset::Error),
}
