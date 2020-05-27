use crate::agent::agent_error::AgentError;
use crate::identity::Identity;
use crate::{Blob, Principal, RequestId, Signature};

pub(crate) struct DummyIdentity {}

impl Identity for DummyIdentity {
    fn sender(&self) -> Result<Principal, AgentError> {
        Ok(Principal::anonymous())
    }

    fn sign(&self, _request: &RequestId, _principal: &Principal) -> Result<Signature, AgentError> {
        Ok(Signature {
            signature: Blob::from(vec![1; 32]),
            public_key: Blob::from(vec![2; 32]),
        })
    }
}
