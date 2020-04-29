//! Provides identity management and operations for the Internet
//! Computer (IC). Namely, we generate, load and revoke credentials
//! related to principals, provide principal mapping seamlessly with
//! corresponding key-pairs.
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
//!
//! A `role` represents the set of actions an entity equipped with that
//! role can exercise.
//!
//! An `identifier` is a sequence of bytes/string utilized as a name for
//! a principal. That allows a principal to be referenced.
//!
//! A `controller` is a principal with an administrative-control role
//! over a corresponding canister. Each canister has one or more
//! controllers.

use crate::crypto_error::Error;
use crate::crypto_error::Result;
use crate::provider::BasicProvider;
use crate::types::Signature;

use std::path::PathBuf;

/// An identity is a construct that denotes the set of claims of an
/// entity about itself. Namely it collects principals, under which
/// the owner of this object can authenticate and provides basic
/// operations. Thus, an identity may have access to multiple
/// principals or credential services, each combination represented by
/// a provider.
pub struct Identity {
    inner: BasicProvider,
}

impl Identity {
    /// Return a corresponding provided a profile path. We pass a
    /// simple configuration for now, but this might change in the
    /// future.
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
