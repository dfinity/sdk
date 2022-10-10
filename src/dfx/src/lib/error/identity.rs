use crate::lib::error::DfxError;
use std::boxed::Box;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IdentityError {
    #[error("Identity already exists.")]
    IdentityAlreadyExists(),

    #[error("An Identity named {0} cannot be created as it is reserved for internal use.")]
    ReservedIdentityName(String),

    #[error("Identity {0} does not exist at '{1}'.")]
    IdentityDoesNotExist(String, PathBuf),

    #[error("Cannot create identity directory at '{0}': {1:#}")]
    CannotCreateIdentityDirectory(PathBuf, Box<DfxError>),

    #[error("Cannot rename identity directory from '{0}' to '{1}': {2:#}")]
    CannotRenameIdentityDirectory(PathBuf, PathBuf, Box<DfxError>),

    #[error("Cannot delete the default identity.")]
    CannotDeleteDefaultIdentity(),

    #[error("Cannot create an anonymous identity.")]
    CannotCreateAnonymousIdentity(),

    #[error("Cannot find home directory.")]
    CannotFindHomeDirectory(),

    #[error("Cannot read identity file '{0}': {1:#}")]
    CannotReadIdentityFile(String, Box<DfxError>),
}
