use crate::lib::error::DfxError;
use ic_identity_hsm::HardwareIdentityError;

use ring::error::Unspecified;
use std::boxed::Box;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IdentityError {
    #[error("Identity already exists.")]
    IdentityAlreadyExists(),

    #[error("Identity {0} does not exist at '{1}'.")]
    IdentityDoesNotExist(String, PathBuf),

    #[error("Cannot generate key pair.")]
    CannotGenerateKeyPair(Unspecified),

    #[error("Cannot create identity directory at '{0}': {1}")]
    CannotCreateIdentityDirectory(PathBuf, Box<DfxError>),

    #[error("Cannot rename identity directory from '{0}' to '{1}': {2}")]
    CannotRenameIdentityDirectory(PathBuf, PathBuf, Box<DfxError>),

    #[error("Cannot delete the default identity.")]
    CannotDeleteDefaultIdentity(),

    #[error("Cannot create an anonymous identity.")]
    CannotCreateAnonymousIdentity(),

    #[error("Cannot find home directory.")]
    CannotFindHomeDirectory(),

    #[error("Cannot read identity file at '{0}': {1}")]
    CannotReadIdentityFile(PathBuf, Box<DfxError>),
}
