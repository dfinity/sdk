use ic_http_agent::to_request_id;
use ic_http_agent::AgentError;
use ic_http_agent::Blob;
use ic_http_agent::RequestId;
use ic_http_agent::SignedMessage;
use ic_http_agent::Signer;
use std::path::PathBuf;

pub struct Identity(ic_identity_manager::Identity);

impl Identity {
    /// Construct a new identity handling object, providing given
    /// configuration.
    pub fn new(identity_config_path: PathBuf) -> Self {
        Self(
            // We panic as discussed, as that should not be the
            // case. I personally prefer this to be an error.
            ic_identity_manager::Identity::new(identity_config_path)
                .expect("Expected a valid identity configuration"),
        )
    }
}

impl Signer for Identity {
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
        Ok((request_id, Box::new(signed_request)))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;
    use serde::Serialize;
    use tempfile::tempdir;

    // Dummy proptest checking request id is correct for now.
    proptest! {
    #[test]
    fn request_id_identity(request: String) {
        let dir = tempdir().unwrap();

        #[derive(Clone,Serialize)]
        struct TestAPI { inner : String}
        let request = TestAPI { inner: request};

        let request_with_sender = request.clone();
        let actual_request_id = to_request_id(&request_with_sender).expect("Failed to produce request id");

        let signer = Identity::new(dir.into_path());
        let request_id = signer.sign(Box::new(request)).expect("Failed to sign").0;
        assert_eq!(request_id, actual_request_id)
    }}
}
