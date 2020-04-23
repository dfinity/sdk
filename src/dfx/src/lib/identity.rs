use ic_agent::to_request_id;
use ic_agent::AgentError;
use ic_agent::Signer;
use ic_agent::{Blob, Request, RequestId, SignedMessage};
use std::path::PathBuf;

pub struct Identity(ic_identity_manager::Identity);

impl Identity {
    /// Construct a new identity handling object, providing given
    /// configuration.
    pub fn new(identity_config_path: PathBuf) -> Self {
        Self(
            // We expect an identity profile to be provided.
            ic_identity_manager::Identity::new(identity_config_path)
                .expect("Expected a valid identity configuration"),
        )
    }
}

impl Signer for Identity {
    fn sign<'a>(&self, request: Request<'a>) -> Result<(RequestId, SignedMessage<'a>), AgentError> {
        let request_id = to_request_id(&request).map_err(AgentError::from)?;
        let signature_tuple = self
            .0
            .sign(Blob::from(request_id).as_slice())
            .map_err(|e| AgentError::SigningError(e.to_string()))?;
        let signature = Blob::from(signature_tuple.signature.clone());
        let sender_pubkey = Blob::from(signature_tuple.public_key);
        let signed_request = SignedMessage {
            request_with_sender: request,
            signature,
            sender_pubkey,
        };
        Ok((request_id, signed_request))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ic_agent::{Blob, CanisterId, ReadRequest};
    use proptest::prelude::*;
    use tempfile::tempdir;

    proptest! {
    #[test]
    fn request_id_identity(request_body: String) {
        let dir = tempdir().unwrap();
        let arg = Blob::from(vec![4; 32]);
        let canister_id = CanisterId::from(Blob::from(vec![4; 32]));
        let request = ReadRequest::Query {
            arg: &arg,
            canister_id: &canister_id,
            method_name: &request_body,
        };

        let request = Request::Query(request.clone());
        let request_with_sender = request.clone();
        let actual_request_id = to_request_id(&request).expect("Failed to produce request id");

        let signer = Identity::new(dir.into_path());
        let request_id = signer.sign(request_with_sender).expect("Failed to sign").0;
        assert_eq!(request_id, actual_request_id)
    }}
}
