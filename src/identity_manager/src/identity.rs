use crate::crypto_error::Error;
use crate::crypto_error::Result;
use crate::provider::basic::BasicProvider;
use crate::provider::Provider;
use crate::signature::Signature;

use std::path::PathBuf;

/// An identity is a construct that denotes the set of claims of an
/// entity about itself.

/// Identification is the procedure whereby an entity claims a certain
/// identity, while verification is the procedure whereby that claim
/// is checked. Authentication is the assertion of an entity’s claim
/// to an identity.

/// A role represents the set of actions an entity equipped with that
/// role can exercise.

/// A principal describes the security context of an identity, namely
/// any identity that can be authenticated along with a specific
/// role. In the case of the Internet Computer this maps currently to
/// the identities that can be authenticated by a canister.

/// A controller is a principal with an administrative-control role
/// over a corresponding canister. Each canister has one or more
/// controllers. A controller can be a person, an organization, or
/// another canister

/// An identifier is a sequence of bytes/string utilized as a name for
/// a principal. That allows a principal to be referenced.

/// An identity describes a user or any entity in general that can be
/// authenticated. An identity may have access to multiple principals
/// or credential services, each combination represented by a provider.
pub struct Identity {
    // TODO(eftychis): This changes into a precendence map. Note that
    // in the future Principals are not going to be tied necessarily
    // with Identifiers from a canister's perspective.
    inner: Vec<Box<dyn Provider>>,
}

impl Identity {
    // Passing a simple configuration until we know all the necessary
    // configuration.
    pub fn new(path: PathBuf) -> Result<Self> {
        let basic_provider = BasicProvider::new(path)?;
        Ok(Self {
            inner: vec![Box::new(basic_provider)],
        })
    }

    pub fn sign(&self, msg: &[u8]) -> Result<Signature> {
        let provider = self.inner.first().ok_or(Error::NoProvider)?;
        let identity = provider
            .provide()
            .map_err(|_| Error::IdentityFailedToInitialize)?;
        identity.sign(msg)
    }
}
