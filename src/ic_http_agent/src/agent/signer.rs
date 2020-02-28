use crate::agent::agent_error::AgentError;
use crate::agent::replica_api::{MessageWithSender, SignedMessage};
use crate::types::request_id::to_request_id;
use crate::{Blob, RequestId};

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
    fn sign<'a>(
        &self,
        request: Box<(dyn erased_serde::Serialize + Send + Sync + 'a)>,
    ) -> Result<
        (
            RequestId,
            Box<dyn erased_serde::Serialize + Send + Sync + 'a>,
        ),
        AgentError,
    >;
}

pub struct DummyIdentity {}

// Right now serialize can not be made into a trait object out of the
// box because of object safety. This should change in the future. For
// the same reason equipping the Signer with a generic function ends
// up in a trait that can not be made into a trait object at compile
// time that depends on a trait with a similar ailment. This makes
// things simply complicated. Making the Signer parametric on a
// Serialize type means we have to pass it along and pushes the issue
// to dfx or the agent main body of code. Thus, we simply treat the
// issue here at its root: we pick an erased Serde Serialize trait and
// return one too. This is compatible with serde Serialize,
// constructing a holder object and intermediate trait in the
// process. Doing it this manually here ends up being messy and
// distracts from the logic. Thus, we use the erased_serde crate.
impl Signer for DummyIdentity {
    fn sign<'a>(
        &self,
        request: Box<(dyn erased_serde::Serialize + Send + Sync + 'a)>,
    ) -> Result<
        (
            RequestId,
            Box<dyn erased_serde::Serialize + Send + Sync + 'a>,
        ),
        AgentError,
    > {
        let mut sender = vec![0; 32];
        sender.push(0x02);
        let sender = Blob::from(sender);
        let request_with_sender = MessageWithSender { request, sender };
        let request_id = to_request_id(&request_with_sender).map_err(AgentError::from)?;

        let signature = Blob::from(vec![1; 32]);
        let sender_pubkey = Blob::from(vec![2; 32]);
        let signed_request = SignedMessage {
            request_with_sender,
            signature,
            sender_pubkey,
        };
        Ok((request_id, Box::new(signed_request)))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;
    use serde::Serialize;

    // TODO(eftychis): Provide arbitrary strategies for the replica
    // API.
    proptest! {
    #[test]
    fn request_id_dummy_signer(request: String) {
        let mut sender = vec![0; 32];
        sender.push(0x02);
        let sender = Blob::from(sender);
        #[derive(Clone,Serialize)]
        struct TestAPI { inner : String}
        let request = TestAPI { inner: request};

        let request_with_sender = MessageWithSender { request:request.clone(), sender };
        let actual_request_id = to_request_id(&request_with_sender).expect("Failed to produce request id");
        let signer = DummyIdentity {};
        let request_id = signer.sign(Box::new(request)).expect("Failed to sign").0;
        assert_eq!(request_id, actual_request_id)
    }}
}
