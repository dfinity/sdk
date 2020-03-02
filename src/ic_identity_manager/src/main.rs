use ic_identity_manager::crypto_error::Result;
use ic_identity_manager::identity::Identity;
use ic_identity_manager::principal::Principal;
use ring::signature::{self, KeyPair};
use std::env::{args, current_dir};
use std::fs;

fn main() -> Result<()> {
    let args: Vec<_> = args().collect();
    let def = &"Hello World!! This is an example test".to_owned();
    let msg = args.first().unwrap_or(def);
    let message = msg.as_str();

    let pwd = current_dir()?;
    let identity = Identity::new(pwd.clone())?;
    let signed_message = identity.sign(message.as_bytes())?;
    println!("Signing {:?}", message.as_bytes());
    let mut path = pwd;
    path.push("creds.pem");
    let pkcs8_bytes = pem::parse(fs::read(path).unwrap()).unwrap().contents;
    let key_pair = signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())?;
    let sig = key_pair.sign(message.as_bytes());
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
    peer_public_key.verify(message.as_bytes(), sig.as_ref())?;
    println!("Signing {:?}", message.as_bytes());
    println!("with key {:?}", peer_public_key_bytes);

    Ok(())
}
