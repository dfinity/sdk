use crate::identity::Signer;
use crate::{AgentError, Blob, Principal, RequestId, Signature};

pub struct DummyIdentity {}

impl Signer for DummyIdentity {
    fn sender(&self) -> Principal {
        // 2 for self authenticating.
        Principal::from(&[
            1u8, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 2,
        ] as &[u8])
    }

    fn sign(&self, _request: &RequestId) -> Result<Signature, AgentError> {
        let sender_sig = Blob::from(vec![1; 32]);
        let sender_pubkey = Blob::from(vec![2; 32]);

        Ok(Signature {
            public_key: sender_pubkey,
            signature: sender_sig,
        })
    }
}
