use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RequireIdentityExistsError {
    #[error("Identity {0} does not exist at '{1}'.")]
    IdentityDoesNotExist(String, PathBuf),

    #[error("An Identity named {0} cannot be created as it is reserved for internal use.")]
    ReservedIdentityName(String),
}
