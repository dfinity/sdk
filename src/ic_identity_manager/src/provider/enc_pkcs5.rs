//! Provides a basic example provider that utilizes unencrypted PEM
//! files. This is provided as a basic stepping stone to provide
//! further functionality.
//!
//! We do not use PKCS#5 as it is outdated. We instead encrypt data at
//! rest using ChaCha20.
use crate::crypto_error::{Error, ParsedKeyError, Result};
use crate::encryption::{decrypt, derive_key, encrypt};
use crate::types::Signature;

use ic_http_agent::Principal;
use pem::{encode, Pem};
use ring::signature::Ed25519KeyPair;
use ring::{
    rand,
    signature::{self, KeyPair},
};
use std::fs;
use std::path::{Path, PathBuf};

// This module should not be re-exported. We want to ensure
// construction and handling of keys is done only here.
use self::private::EncryptedKeyProviderReady;

#[derive(Clone)]
pub struct EncryptedKeyProvider {
    path: PathBuf,
    passphrase: String,
}

impl EncryptedKeyProvider {
    pub fn new(path: PathBuf, passphrase: String) -> Result<Self> {
        if !path.is_dir() {
            return Err(Error::ProviderFailedToInitialize);
        }
        Ok(Self { path, passphrase })
    }
}

fn generate(profile_path: &impl AsRef<Path>, passphrase: &[u8]) -> Result<PathBuf> {
    let rng = rand::SystemRandom::new();
    let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng)?;
    let key = derive_key(passphrase, &[0u8])?;
    let pkcs8_bytes = encrypt(&(*pkcs8_bytes.as_ref()), key.as_ref())?;
    let pem_file = profile_path.as_ref().join("creds.pem");
    let contents = encode_pem_private_key(&pkcs8_bytes);
    fs::write(&pem_file, contents)?;

    assert_eq!(pem::parse(fs::read(&pem_file)?)?.contents, pkcs8_bytes);
    Ok(pem_file)
}

impl EncryptedKeyProvider {
    pub fn provide(&self) -> Result<EncryptedKeyProviderReady> {
        let mut dir = fs::read_dir(&self.path)?;
        let name: std::ffi::OsString = "creds.pem".to_owned().into();
        let pem_file = if dir.any(|n| match n {
            Ok(n) => n.file_name() == name,
            Err(_) => false,
        }) {
            self.path.join("creds.pem")
        } else {
            generate(&self.path, self.passphrase.as_bytes())?
        };

        let pem_value = pem::parse(fs::read(pem_file)?)?;
        if "ENCRYPTED PRIVATE KEY" != pem_value.tag {
            return Err(Error::ParsedKeyError(ParsedKeyError::PrivateKeyPlaintext));
        }

        let pkcs8_bytes = pem_value.contents;
        let key = derive_key(&self.passphrase.as_bytes(), &[0u8])?;
        let pkcs8_bytes = decrypt(&pkcs8_bytes, &key)?;

        let key_pair = signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())?;

        Ok(EncryptedKeyProviderReady { key_pair })
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
    pub struct EncryptedKeyProviderReady {
        pub key_pair: Ed25519KeyPair,
    }

    impl EncryptedKeyProviderReady {
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
        fn principal(&self) -> Principal {
            Principal::self_authenticating(&self.key_pair.public_key())
        }
    }
}

fn encode_pem_private_key(key: &[u8]) -> String {
    let pem = Pem {
        tag: "ENCRYPTED PRIVATE KEY".to_owned(),
        contents: key.to_vec(),
    };
    encode(&pem)
}
