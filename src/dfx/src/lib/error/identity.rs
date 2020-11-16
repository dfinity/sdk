use ring::error::Unspecified;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IdentityError {
    #[error("Identity already exists.")]
    IdentityAlreadyExists(),

    #[error("Identity '{0}' does not exist at '{1}'.")]
    IdentityDoesNotExist(String, PathBuf),

    #[error("Cannot generate key pair.")]
    CannotGenerateKeyPair(Unspecified),

    #[error("Cannot create identity directroy at '{0}'.")]
    CannotCreateIdentityDirectory(PathBuf),

    #[error("Cannot rename identity directroy from '{0}' to '{1}'.")]
    CannotRenameIdentityDirectory(PathBuf, PathBuf),

    #[error("Cannot delete the default identity.")]
    CannotDeleteDefaultIdentity(),

    #[error("Cannot create an anonymous identity.")]
    CannotCreateAnonymousIdentity(),

    #[error("Cannot find home directory.")]
    CannotFindHomeDirectory(),

    #[error("Cannot read identity file at '{0}'")]
    CannotReadIdentityFile(PathBuf),
}
