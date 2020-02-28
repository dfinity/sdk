//! A user may have multiple principal&credential providers.

use crate::crypto_error::Result;
use crate::principal::Principal;
use crate::signature::Signature;

/// Keeps track and provides an IdentityWallet to allow a user to
/// authenticate with a particular principal and service services. We
/// do not necessarily keep in memory related credentials, but can
/// reach out to a third-party service to provide us with the
/// principal and the signing functionality.
pub trait Provider {
    fn provide(&self) -> Result<Box<dyn IdentityWallet>>;
}

/// Provide access to a signing functionality to represent a
/// particular principal.
pub trait IdentityWallet {
    fn sign(&self, msg: &[u8]) -> Result<Signature>;
    fn principal(&self) -> Principal;
}

// Keeping it simple and public right now.
pub mod basic;
