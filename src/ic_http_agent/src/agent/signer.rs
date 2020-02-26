use crate::{Blob, RequestId};

/// Represents the signature which accompanies a request.
#[allow(dead_code)]
pub struct Signature {
    sender: Blob,
    signature: Blob,
    sender_pubkey: Blob,
}

/// A Signer provides a [Signature] for the given [RequestId].
pub trait Signer {
    fn signature(request_id: RequestId) -> Signature;
}

// Used for testing purposes only.
#[cfg(test)]
#[allow(dead_code)]
struct DummyIdentity {}

#[cfg(test)]
impl Signer for DummyIdentity {
    fn signature(_: RequestId) -> Signature {
        Signature {
            sender: Blob::from(vec![0; 33]),
            signature: Blob::from(vec![1; 32]),
            sender_pubkey: Blob::from(vec![2; 32]),
        }
    }
}
