use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use super::identity_manager::EncryptionConfiguration;

use aes_gcm::aead::{Aead, NewAead};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use anyhow::anyhow;
use argon2::{password_hash::PasswordHasher, Argon2};
use ic_types::principal::Principal;

#[derive(Debug, PartialEq)]
pub enum CallSender {
    SelectedId,
    Wallet(Principal),
}

// Determine whether the selected Identity
// or the provided wallet canister ID should be the Sender of the call.
pub async fn call_sender(_env: &dyn Environment, wallet: &Option<String>) -> DfxResult<CallSender> {
    let sender = if let Some(id) = wallet {
        CallSender::Wallet(Principal::from_text(&id)?)
    } else {
        CallSender::SelectedId
    };
    Ok(sender)
}

fn get_argon_params() -> argon2::Params {
    argon2::Params::new(64000 /* in kb */, 3, 1, Some(32 /* in bytes */)).unwrap()
}

pub fn encrypt(
    content: &[u8],
    config: &EncryptionConfiguration,
    password: &str,
) -> DfxResult<Vec<u8>> {
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        get_argon_params(),
    );
    let hash = argon2
        .hash_password(password.as_bytes(), &config.pw_salt)
        .map_err(|e| anyhow!(format!("Error during password hashing: {}", e)))?;
    let key = Key::clone_from_slice(hash.hash.unwrap().as_ref());
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(config.file_nonce.as_slice());

    let encrypted = cipher
        .encrypt(nonce, content)
        .map_err(|_| anyhow!("Encryption failed."))?;

    Ok(encrypted)
}

pub fn decrypt(
    encrypted_content: &[u8],
    config: &EncryptionConfiguration,
    password: &str,
) -> DfxResult<Vec<u8>> {
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        get_argon_params(),
    );
    let hash = argon2
        .hash_password(password.as_bytes(), &config.pw_salt)
        .map_err(|e| anyhow!(format!("Error during password hashing: {}", e)))?;
    let key = Key::clone_from_slice(hash.hash.unwrap().as_ref());
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(config.file_nonce.as_slice());

    let decrypted = cipher
        .decrypt(nonce, encrypted_content.as_ref())
        .map_err(|_| anyhow!("Decryption failed."))?;
    Ok(decrypted)
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(5))] // takes ~10s per case
        #[test]
        fn decrypt_reverts_encrypt(pass in ".*", content in ".*") {
            let config = EncryptionConfiguration::new().unwrap();
            let encrypted = encrypt(content.as_bytes(), &config, &pass).unwrap();
            let decrypted = decrypt(&encrypted, &config, &pass).unwrap();

            assert_eq!(content.as_bytes(), decrypted.as_slice());
        }
    }
}
