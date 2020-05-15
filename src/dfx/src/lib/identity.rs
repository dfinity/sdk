use ic_agent::Principal;
use ic_agent::{AgentError, Signature};
use ic_agent::{Blob, RequestId};
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

impl ic_agent::Identity for Identity {
    fn sender(&self) -> Result<Principal, AgentError> {
        Ok(self.0.sender())
    }

    fn sign(&self, request_id: &RequestId, _: &Principal) -> Result<Signature, AgentError> {
        let signature_tuple = self
            .0
            .sign(Blob::from(*request_id).as_slice())
            .map_err(|e| AgentError::SigningError(e.to_string()))?;

        let signature = Blob::from(signature_tuple.signature.clone());
        let public_key = Blob::from(signature_tuple.public_key);
        Ok(Signature {
            public_key,
            signature,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ic_agent::Identity;
    use tempfile::tempdir;

    #[test]
    fn request_id_identity() {
        let dir = tempdir().unwrap();
        let request_id = RequestId::new(&[4; 32]);

        let signer = super::Identity::new(dir.into_path());
        let sender = signer.sender().expect("Failed to get the sender.");
        let signature = signer.sign(&request_id, &sender).expect("Failed to sign.");

        // Assert the principal is used for the public key.
        assert_eq!(
            sender,
            Principal::self_authenticating(signature.public_key.as_slice())
        );
    }
}
