use ic_http_agent::{
    to_request_id, AgentError, Blob, MessageWithSender, PrincipalId, Request, RequestId,
    SignedMessage, Signer,
};
use ring::signature::{Ed25519KeyPair, KeyPair};
use std::path::Path;

mod lib_test;

#[derive(Debug)]
pub enum PemIdentityError {
    IoError(std::io::Error),
    PemError(pem::PemError),
    KeyRejected(String),
    RingError(String),
}

#[derive(Debug)]
pub struct PemIdentity {
    pub(crate) principal_id: PrincipalId,
    key_pair: Ed25519KeyPair,
}

impl PemIdentity {
    pub fn from_file(path: &Path) -> Result<Self, PemIdentityError> {
        let pem = pem::parse(std::fs::read(path).map_err(PemIdentityError::IoError)?)
            .map_err(PemIdentityError::PemError)?;
        let key_pair = Ed25519KeyPair::from_pkcs8(&pem.contents)
            .map_err(|x| PemIdentityError::KeyRejected(x.to_string()))?;

        let principal_id = PrincipalId::self_authenticating(Blob::from(&key_pair.public_key()));

        Ok(PemIdentity {
            key_pair,
            principal_id,
        })
    }

    pub fn generate(pem_file: &Path) -> Result<(), PemIdentityError> {
        let rng = ring::rand::SystemRandom::new();
        let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng)
            .map_err(|x| PemIdentityError::RingError(x.to_string()))?;
        // We create a temporary file that gets overwritten every time
        // we create a new provider for now.
        let pem = pem::Pem {
            tag: "PRIVATE KEY".to_owned(),
            contents: pkcs8_bytes.as_ref().to_vec(),
        };

        std::fs::write(pem_file, pem::encode(&pem)).map_err(PemIdentityError::IoError)?;
        Ok(())
    }
}

impl Signer for PemIdentity {
    fn sign<'a>(&self, request: Request<'a>) -> Result<(RequestId, SignedMessage<'a>), AgentError> {
        let request_with_sender = MessageWithSender {
            request,
            sender: self.principal_id.clone(),
        };
        let message_cbor =
            serde_cbor::to_vec(&request_with_sender).map_err(AgentError::SerdeError)?;
        let request_id = to_request_id(&request_with_sender)?;

        let signature = self.key_pair.sign(&message_cbor);
        Ok((
            request_id,
            SignedMessage {
                request_with_sender,
                sender_pubkey: Blob::from(&self.key_pair.public_key()),
                sender_sig: Blob::from(&signature),
            },
        ))
    }
}
