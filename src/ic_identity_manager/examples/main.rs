use identity_manager::crypto_error::Result;
use identity_manager::identity::Identity;
use identity_manager::principal::Principal;
use ring::signature::{self, KeyPair};
use std::env::current_dir;
use std::fs;

fn main() -> Result<()> {
    const MESSAGE: &[u8] = b"Hello World!! This is an example test";
    let pwd = current_dir()?;
    let identity = Identity::new(pwd)?;
    let signed_message = identity.sign(MESSAGE)?;

    let pkcs8_bytes = pem::parse(fs::read("creds.pem").unwrap()).unwrap().contents;
    let key_pair = signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())?;
    let sig = key_pair.sign(MESSAGE);
    assert_eq!(sig.as_ref().to_vec(), signed_message.signature);
    assert_eq!(
        Principal::self_authenticating(&key_pair),
        signed_message.signer
    );
    assert_eq!(
        key_pair.public_key().as_ref().to_vec(),
        signed_message.public_key
    );

    let peer_public_key_bytes = key_pair.public_key().as_ref();
    let peer_public_key =
        signature::UnparsedPublicKey::new(&signature::ED25519, peer_public_key_bytes);
    peer_public_key.verify(MESSAGE, sig.as_ref())?;

    let signed_message_2 = identity.sign(MESSAGE)?;
    assert_eq!(*signed_message_2.signature, *signed_message.signature);
    assert_eq!(signed_message_2.public_key, signed_message.public_key);
    assert_eq!(signed_message_2.signer, signed_message.signer);

    Ok(())
}
