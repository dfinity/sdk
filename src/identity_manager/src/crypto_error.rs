use std::{error, result};

#[derive(Debug)]
pub enum Error {
    /// A CryptoError is isomorphic to unit on purpose. In case of
    /// such a failure, we establish a new This is nice as Rust is
    /// eager in general so we don't have to worry about lazy
    /// evaluation of errors.
    CryptoError,
    NoProvider,
    IdentityFailedToInitialize,
    IOError(std::io::Error),
}

impl From<ring::error::Unspecified> for Error {
    fn from(_: ring::error::Unspecified) -> Self {
        Error::CryptoError
    }
}

impl From<ring::error::KeyRejected> for Error {
    fn from(_: ring::error::KeyRejected) -> Self {
        Error::CryptoError
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IOError(e)
    }
}

impl error::Error for Error {
    // We do not need source for now.
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Currently Display = Debug for all intents and purposes.
        write!(fmt, "{:?}", self)
    }
}

pub type Result<T> = result::Result<T, Error>;
