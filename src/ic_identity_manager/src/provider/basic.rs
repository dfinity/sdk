use crate::crypto_error::Result;
use crate::principal::Principal;
use crate::provider::{IdentityWallet, Provider};
use crate::signature::Signature;

use pem::{encode, Pem};
use ring::signature::Ed25519KeyPair;
use ring::{
    rand,
    signature::{self, KeyPair},
};
use std::fs;
use std::path::PathBuf;

#[derive(Clone)]
pub struct BasicProvider {
    path: PathBuf,
}

impl BasicProvider {
    pub fn new(path: PathBuf) -> Result<Self> {
        generate(path.clone())?;
        Ok(Self { path })
    }
}

fn generate(mut pem_file: PathBuf) -> Result<()> {
    let rng = rand::SystemRandom::new();
    let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng)?;
    // We create a temporary file that gets overwritten every time
    // we create a new provider for now.
    pem_file.push("creds.pem");
    fs::write(
        pem_file.clone(),
        encode_pem_private_key(&(*pkcs8_bytes.as_ref())),
    )?;

    assert_eq!(
        pem::parse(fs::read(pem_file)?)?.contents,
        pkcs8_bytes.as_ref()
    );
    Ok(())
}

struct BasicProviderReady {
    key_pair: Ed25519KeyPair,
}

impl Provider for BasicProvider {
    fn provide(&self) -> Result<Box<dyn IdentityWallet>> {
        let mut pem_file = self.path.clone();
        pem_file.push("creds.pem");

        let pkcs8_bytes = pem::parse(fs::read(pem_file)?)?.contents;
        let key_pair = signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())?;

        Ok(Box::new(BasicProviderReady { key_pair }))
    }
}

impl IdentityWallet for BasicProviderReady {
    fn sign(&self, msg: &[u8]) -> Result<Signature> {
        let signature = self.key_pair.sign(msg);
        // At this point we shall validate the signature in this first
        // skeleton version.
        let public_key_bytes = self.key_pair.public_key().as_ref();

        let public_key = signature::UnparsedPublicKey::new(&signature::ED25519, public_key_bytes);
        public_key.verify(msg, signature.as_ref())?;
        Ok(Signature {
            signer: self.principal(),
            signature: signature.as_ref().to_vec(),
            public_key: public_key_bytes.to_vec(),
        })
    }
    fn principal(&self) -> Principal {
        Principal::self_authenticating(&self.key_pair)
    }
}

fn encode_pem_private_key(key: &[u8]) -> String {
    let pem = Pem {
        tag: "PRIVATE KEY".to_owned(),
        contents: key.to_vec(),
    };
    encode(&pem)
}
