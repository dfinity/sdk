use crate::agent::agent_error::AgentError;
use crate::agent::replica_api::{Request, SignedMessage};
use crate::types::request_id::to_request_id;
use crate::{Blob, MessageWithSender, RequestId};

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
    fn sign<'a>(&self, request: Request<'a>) -> Result<(RequestId, SignedMessage<'a>), AgentError>;
}

pub struct DummyIdentity {}

impl Signer for DummyIdentity {
    fn sign<'a>(&self, content: Request<'a>) -> Result<(RequestId, SignedMessage<'a>), AgentError> {
        let request_with_sender = MessageWithSender {
            content,
            sender: Blob::from(vec![1; 32]),
        };
        let request_id = to_request_id(&request_with_sender).map_err(AgentError::from)?;

        let signature = Blob::from(vec![1; 32]);
        let sender_pubkey = Blob::from(vec![2; 32]);
        let signed_request = SignedMessage {
            request_with_sender,
            sender_sig: signature,
            sender_pubkey,
        };
        Ok((request_id, signed_request))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::agent::replica_api::{ReadRequest, Request};
    use crate::CanisterId;

    use proptest::prelude::*;

    // TODO(eftychis): Provide arbitrary strategies for the replica
    // API.
    proptest! {
    #[test]
    fn request_id_dummy_signer(request_body: String) {
        let arg = Blob::random(10);
        let canister_id = CanisterId::from(Blob::random(10));
        let request = ReadRequest::Query {
            arg: &arg,
            canister_id: &canister_id,
            method_name: &request_body,
        };



        let request_with_sender = Request::Query(request.clone());
        let actual_request_id = to_request_id(&request_with_sender).expect("Failed to produce request id");
        let signer = DummyIdentity {};
        let request_id = signer.sign(request_with_sender).expect("Failed to sign").0;
        assert_eq!(request_id, actual_request_id)
    }}
}
