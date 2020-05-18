//! Provides a basic example provider that utilizes unencrypted PEM
//! files. This is provided as a basic stepping stone to provide
//! further functionality. Note that working with unencrypted PEM is
//! not the best idea.
//!
//! However, there are two options: i) prompt the user per call, as
//! the agent is "stateless" or ii) provide long-running service
//! providers -- such as PGP, ssh-agent.
use crate::crypto_error::{Error, Result};
use crate::file_hierarchy::{FileHierarchy, ProfileIdentifier, UserProfile};
use crate::types::Signature;

use ic_agent::Principal;
use pem::{encode, Pem};
use ring::signature::Ed25519KeyPair;
use ring::{
    rand,
    signature::{self, KeyPair},
};
use std::fs;
use std::path::PathBuf;

// This module should not be re-exported. We want to ensure
// construction and handling of keys is done only here.
use self::private::BasicSignerReady;

#[derive(Clone)]
pub struct BasicSigner {
    path: PathBuf,
}

impl BasicSigner {
    pub fn new(path: PathBuf) -> Result<Self> {
        if !path.is_dir() {
            return Err(Error::ProviderFailedToInitialize);
        }
        Ok(Self { path })
    }
}

fn generate() -> Result<String> {
    let rng = rand::SystemRandom::new();
    let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng)?;
    let pem = encode_pem_private_key(&(*pkcs8_bytes.as_ref()));
    Ok(pem)
}

impl BasicSigner {
    pub fn provide(&self) -> Result<BasicSignerReady> {
        let profile_name = "default";
        let profile_id = ProfileIdentifier::new(profile_name);
        let root = self.path.clone();
        // Ensure there is a setup.
        FileHierarchy::new(root.clone()).setup()?;

        let pem_file =
            match FileHierarchy::partial_load_file_hierarchy(&[profile_id.clone()], &root) {
                Ok(fh) => fh
                    .inner
                    .get(&profile_id)
                    .cloned()
                    .ok_or_else(|| Error::ProfileMissing(profile_id.clone())),
                Err(_) => {
                    let fh = FileHierarchy::new(root.clone());
                    let key = generate()?;
                    let profile = UserProfile::new_with_key(profile_name, key, "main_key");
                    fh.add_profile(profile)?;
                    let fh =
                        FileHierarchy::partial_load_file_hierarchy(&[profile_id.clone()], &root)?;
                    fh.inner
                        .get(&profile_id)
                        .cloned()
                        .ok_or_else(|| Error::ProfileMissing(profile_id.clone()))
                }
            }?
            .get_default_file(profile_id.clone())
            // We have the realtive path; now return the absolute path.
            .map(|r_path| root.join(r_path.path))
            .ok_or_else(|| Error::ProfileMissing(profile_id))?;

        let pkcs8_bytes = pem::parse(fs::read(pem_file)?)?.contents;
        let key_pair = signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())?;

        Ok(BasicSignerReady { key_pair })
    }
}

// The contents of this module while public, that is can be known and
// handled as of the new Rust iteration by other modules in the crate,
// the type constructor and associated functions shall be visible only
// by the parent module, and should not be re-exported. This is
// essentially a sealed type.
mod private {
    use super::*;
    /// We enforce a state transition, reading the key as necessary, only
    /// to sign. TODO(eftychis): We should erase pin and erase the key
    /// from memory afterwards.
    pub struct BasicSignerReady {
        pub key_pair: Ed25519KeyPair,
    }

    impl BasicSignerReady {
        pub fn sign(&self, msg: &[u8]) -> Result<Signature> {
            let signature = self.key_pair.sign(msg);
            // At this point we shall validate the signature in this first
            // skeleton version.
            let public_key_bytes = self.key_pair.public_key().as_ref();

            let public_key =
                signature::UnparsedPublicKey::new(&signature::ED25519, public_key_bytes);
            public_key.verify(msg, signature.as_ref())?;
            Ok(Signature {
                signer: self.principal(),
                signature: signature.as_ref().to_vec(),
                public_key: public_key_bytes.to_vec(),
            })
        }
        pub fn principal(&self) -> Principal {
            Principal::self_authenticating(&self.key_pair.public_key())
        }
    }
}

fn encode_pem_private_key(key: &[u8]) -> String {
    let pem = Pem {
        tag: "PRIVATE KEY".to_owned(),
        contents: key.to_vec(),
    };
    encode(&pem)
}

#[cfg(test)]
mod tests {

    use super::*;

    use tempfile::tempdir;

    #[test]
    fn test_basic_signing() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let provider = BasicSigner::new(root).expect("Failed to construct basic signer");
        provider.provide().unwrap();
        dir.close().unwrap();
    }
}
