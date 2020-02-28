use ic_http_agent::to_request_id;
use ic_http_agent::AgentError;
use ic_http_agent::Blob;
use ic_http_agent::RequestId;
use ic_http_agent::Signer;
use ic_http_agent::{MessageWithSender, SignedMessage};
use std::path::PathBuf;

// This is a stand in for the identity type of the identity manager.
// TODO(eftychis): In Rust 1.41 we will simply add an orphan trait
// here. However, right now we are stuck with rust 1.40 due to dfinity
// repo issues.
pub struct Identity {}

impl Identity {
    /// Construct a new identity handling object, providing given
    /// configuration.
    pub fn new(_: PathBuf) -> Self {
        Self {}
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

    // Dummy proptest checking request id is correct for now.
    proptest! {
    #[test]
    fn request_id_identity(request: String) {
        let mut sender = vec![0; 32];
        sender.push(0x02);
        let sender = Blob::from(sender);
        #[derive(Clone,Serialize)]
        struct TestAPI { inner : String}
        let request = TestAPI { inner: request};

        let request_with_sender = MessageWithSender { request:request.clone(), sender };
        let actual_request_id = to_request_id(&request_with_sender).expect("Failed to produce request id");
        let signer = Identity::new(PathBuf::from(""));
        let request_id = signer.sign(Box::new(request)).expect("Failed to sign").0;
        assert_eq!(request_id, actual_request_id)
    }}
}
