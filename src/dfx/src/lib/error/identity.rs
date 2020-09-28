use ring::error::Unspecified;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IdentityErrorKind {
    #[error("Identity already exists")]
    IdentityAlreadyExists(),

    #[error("Identity {0} does not exist at {1}")]
    IdentityDoesNotExist(String, PathBuf),

    #[error("Could not generate key")]
    CouldNotGenerateKey(Unspecified),

    #[error(r#"Could not create the identity folder at "{0}". Error: {1}"#)]
    CouldNotCreateIdentityDirectory(PathBuf, io::Error),

    #[error(r#"Could not rename identity directory {0} to {1}: {2}"#)]
    CouldNotRenameIdentityDirectory(PathBuf, PathBuf, io::Error),

    #[error("Cannot delete the default identity")]
    CannotDeleteDefaultIdentity(),

    #[error("Cannot create an anonymous identity")]
    CannotCreateAnonymousIdentity(),

    #[error("Cannot find the home directory.")]
    CannotFindUserHomeDirectory(),

    #[error("An error occurred while reading {1}: {0}")]
    AgentPemError(ic_agent::identity::PemError, PathBuf),
}
