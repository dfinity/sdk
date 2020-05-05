pub(crate) mod dummy;

use crate::{AgentError, Principal, RequestId, Signature};

pub(crate) mod public {
    pub use super::dummy::DummyIdentity;
    pub use super::Signer;
}

/// A Signer amends the request with the [Signature] fields, computing
/// the request id in the process.
///
/// # Warnings / Panics
///
/// While the argument type indicates anything serializable, in
/// reality we can only process only anything that can have a request
/// id. If an argument is provided with no derivable request id, the
/// behaviour is undefined and it is left up to the implementation.
// Note: Turning a trait into async at the moment imposes a static
// lifetime, which ends up complicating and polluting the remaining
// code.
pub trait Signer: Sync {
    fn sender(&self) -> Principal;
    fn sign(&self, request: &RequestId) -> Result<Signature, AgentError>;
}
