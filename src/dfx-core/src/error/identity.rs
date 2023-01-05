use ic_agent::identity::PemError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IdentityError {
    #[error("Cannot delete the default identity.")]
    CannotDeleteDefaultIdentity(),

    #[error("Cannot delete the anonymous identity.")]
    CannotDeleteAnonymousIdentity(),

    #[error("Cannot create an anonymous identity.")]
    CannotCreateAnonymousIdentity(),

    #[error("Cannot create identity directory at '{0}': {1:#}")]
    CreateIdentityDirectoryFailed(PathBuf, std::io::Error),

    #[error("Identity already exists.")]
    IdentityAlreadyExists(),

    #[error("Identity {0} does not exist at '{1}'.")]
    IdentityDoesNotExist(String, PathBuf),

    #[error("Cannot find home directory (no HOME environment variable).")]
    NoHomeInEnvironment(),

    #[error("Cannot read identity file '{0}': {1:#}")]
    ReadIdentityFileFailed(String, PemError),

    #[error("Cannot rename identity directory from '{0}' to '{1}': {2:#}")]
    RenameIdentityDirectoryFailed(PathBuf, PathBuf, std::io::Error),

    #[error("An Identity named {0} cannot be created as it is reserved for internal use.")]
    ReservedIdentityName(String),
}
