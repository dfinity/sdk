use crate::{AgentError, Blob, Principal, RequestId};

pub(crate) mod dummy;

pub(crate) mod public {
    pub use super::{Identity, Signature};
}

#[derive(Clone, Debug)]
pub struct Signature {
    pub public_key: Blob,
    pub signature: Blob,
}

/// An Identity takes a request id and returns the [Signature]. Since it
/// also knows about the Principal of the sender.
///
/// Agents are assigned a single Identity object, but there can be multiple
/// identities used
pub trait Identity: Send + Sync {
    /// Returns a sender, ie. the Principal ID that is used to sign a request.
    /// Only one sender can be used per request.
    fn sender(&self) -> Result<Principal, AgentError>;

    /// Sign a concatenation of the domain separator & request ID,
    /// creating the sender signature, with the principal passed in.
    /// The principal should be
    /// the same returned by the call to `sender()`.
    fn sign(
        &self,
        domain_separator: &[u8],
        request: &RequestId,
        principal: &Principal,
    ) -> Result<Signature, AgentError>;
}
