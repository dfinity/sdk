use std::io;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigErrorKind {
    #[error("Cannot find the home directory.")]
    CannotFindUserHomeDirectory(),

    #[error(r#"The configuration folder "{0}" should be a directory."#)]
    HomeConfigDfxShouldBeADirectory(PathBuf),

    #[error(r#"Could not create the configuration folder at "{0}". Error: {1}"#)]
    CouldNotCreateConfigDirectory(PathBuf, io::Error),
}
