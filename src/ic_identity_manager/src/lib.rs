//! # Usage
//!
//! We expose a single type [`Identity`], currently providing only
//! signing and principal.
//!
//! # Examples
//! ```not_run
//! use ic_identity_manager::identity::Identity;
//!
//! let identity =
//! Identity::new(std::path::PathBuf::from("temp_dir")).expect("Failed to construct an identity object");
//! let _signed_message = identity.sign(b"Hello World! This is Bob").expect("Signing failed");
//! ```

/// Provides basic error type and messages.
pub mod crypto_error;

mod basic;
mod types;

use crate::basic::BasicSigner;
use crate::crypto_error::Error;
use crate::crypto_error::Result;
use crate::types::Signature;

use std::path::PathBuf;

/// An identity is a construct that denotes the set of claims of an
/// entity about itself. Namely it collects principals, under which
/// the owner of this object can authenticate and provides basic
/// operations.
pub struct Identity {
    inner: BasicSigner,
}

impl Identity {
    /// Return a corresponding provided a profile path.  We pass a simple
    /// configuration for now, but this might change in the future.
    pub fn new(path: PathBuf) -> Result<Self> {
        let basic_provider = BasicSigner::new(path)?;
        Ok(Self {
            inner: basic_provider,
        })
    }
    /// Sign the provided message assuming a certain principal.
    pub fn sign(&self, msg: &[u8]) -> Result<Signature> {
        let identity = self
            .inner
            .provide()
            .map_err(|_| Error::IdentityFailedToInitialize)?;
        identity.sign(msg)
    }
}
