use crate::crypto_error::{Error, Result};

// use pem::{encode, Pem};
use ring::aead::CHACHA20_POLY1305;
use ring::aead::{Aad, BoundKey, UnboundKey};
use ring::aead::{Nonce, NonceSequence, OpeningKey, SealingKey};
// use ring::signature::Ed25519KeyPair;
use ring::{rand, rand::SecureRandom};
use scrypt::{scrypt, ScryptParams};

struct RNonce;
impl NonceSequence for RNonce {
    fn advance(&mut self) -> std::result::Result<Nonce, ring::error::Unspecified> {
        generate_nonce().map_err(|_| ring::error::Unspecified)
    }
}

fn generate_nonce() -> Result<Nonce> {
    let rng = rand::SystemRandom::new();
    let mut nonce_bytes = [0; 12];
    rng.fill(&mut nonce_bytes).map_err(|_| Error::CryptoError)?;
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);
    Ok(nonce)
}

pub fn decrypt(key: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>> {
    let key = UnboundKey::new(&CHACHA20_POLY1305, key)?;
    let nonce = RNonce;

    let mut key = OpeningKey::new(key, nonce);
    let mut ciphertext = ciphertext.to_vec().clone();
    let plaintext = key.open_in_place(Aad::empty(), &mut ciphertext)?;
    Ok(plaintext.to_vec())
}

pub fn encrypt(key: &[u8], plaintext: &[u8]) -> Result<Vec<u8>> {
    let key = UnboundKey::new(&CHACHA20_POLY1305, key)?;
    let nonce = RNonce;

    let mut key = SealingKey::new(key, nonce);
    let mut plaintext = plaintext.clone().to_vec();
    plaintext.extend(vec![0u8; CHACHA20_POLY1305.tag_len()]);
    key.seal_in_place_append_tag(Aad::empty(), &mut plaintext)?;
    let ciphertext = plaintext;
    // debug_assert_eq!(plaintext.len());

    // plaintext.extend(&raw_nonce);
    Ok(ciphertext)
}

/// We use scrypt for the key derivation.
pub fn derive_key(passphrase: &[u8], salt: &[u8]) -> Result<Vec<u8>> {
    // First setup the ScryptParams arguments with:
    // r = 8, p = 1, n = 32768 (log2(n) = 15)
    let params = ScryptParams::new(16, 8, 1).unwrap();
    // Hash the password for storage
    // let key = scrypt_simple("Not so secure password", &params).expect("OS RNG should not fail");
    let mut key = Vec::new();
    scrypt(passphrase, &salt, &params, &mut key).map_err(|_| Error::CryptoError)?;

    // // const n: i32 = 1 << 20;
    // const n: i32 = 1 << 16;
    // const r: i32 = 8;
    // const p: i32 = 1;
    // let mut key = [0u8];
    // let max_memory = 1 << 14;
    // scrypt(passphrase, salt, n, r, p, max_memory, key)?;
    Ok(key.to_vec())
}
