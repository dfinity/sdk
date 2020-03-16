//! Provides identity related operations for the Internet
//! Computer (IC).
//!
//! # Definitions
//!
//! An [`Identity`] is a construct that denotes the set of claims of an
//! entity about itself.
//!
//! A [`Principal`] describes the security context of an identity, namely
//! any identity that can be authenticated along with a specific
//! role. In the case of the Internet Computer this maps currently to
//! the identities that can be authenticated by a canister.
//!
//! `Identification` is the procedure whereby an entity claims a certain
//! identity, while verification is the procedure whereby that claim
//! is checked. Authentication is the assertion of an entity’s claim
//! to an identity.

use crate::basic::BasicProvider;
use crate::crypto_error::Error;
use crate::crypto_error::Result;
use crate::types::Signature;

use std::path::PathBuf;

/// An identity is a construct that denotes the set of claims of an
/// entity about itself. Namely it collects principals, under which
/// the owner of this object can authenticate and provides basic
/// operations.
pub struct Identity {
    inner: BasicProvider,
}

impl Identity {
    /// Return a corresponding provided a profile path.  We pass a simple
    /// configuration for now, but this might change in the future.
    pub fn new(path: PathBuf) -> Result<Self> {
        let basic_provider = BasicProvider::new(path)?;
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
