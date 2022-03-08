use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use super::identity_manager::EncryptionConfiguration;

// use aes_gcm::aead::{Aead, NewAead};
// use aes_gcm::{Aes256Gcm, Key, Nonce};
// use anyhow::anyhow;
// use argon2;
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

// const ARGON_CONFIG: argon2::Config<'_> = argon2::Config {
//     ad: &[],
//     hash_length: 32,
//     lanes: 1,
//     mem_cost: 4096,
//     secret: &[],
//     thread_mode: argon2::ThreadMode::Sequential,
//     time_cost: 16,
//     variant: argon2::Variant::Argon2id,
//     version: argon2::Version::Version13,
// };

pub fn encrypt(
    content: &[u8],
    _config: &EncryptionConfiguration,
    _password: &str,
) -> DfxResult<Vec<u8>> {
    Ok(Vec::from(content))
    // let key = argon2::hash_raw(
    //     password.as_bytes(),
    //     config.pw_salt.as_slice(),
    //     &ARGON_CONFIG,
    // )?;
    // let key = Key::from_slice(key.as_slice());
    // let cipher = Aes256Gcm::new(key);
    // let nonce = Nonce::from_slice(config.file_nonce.as_slice());

    // let encrypted = cipher
    //     .encrypt(nonce, content)
    //     .map_err(|_| anyhow!("Encryption failed."))?;

    // Ok(encrypted)
}

pub fn decrypt(
    encrypted_content: &[u8],
    _config: &EncryptionConfiguration,
    _password: &str,
) -> DfxResult<Vec<u8>> {
    Ok(Vec::from(encrypted_content))
    // let key = argon2::hash_raw(
    //     password.as_bytes(),
    //     config.pw_salt.as_slice(),
    //     &ARGON_CONFIG,
    // )?;
    // let key = Key::from_slice(key.as_slice());
    // let cipher = Aes256Gcm::new(key);
    // let nonce = Nonce::from_slice(config.file_nonce.as_slice());

    // let decrypted = cipher
    //     .decrypt(nonce, encrypted_content.as_ref())
    //     .map_err(|_| anyhow!("Decryption failed."))?;
    // Ok(decrypted)
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))] // takes ~2.6s per case
        #[test]
        fn decrypt_reverts_encrypt(pass in ".*", content in ".*") {
            let config = EncryptionConfiguration::new().unwrap();
            let encrypted = encrypt(content.as_bytes(), &config, &pass).unwrap();
            let decrypted = decrypt(&encrypted, &config, &pass).unwrap();

            assert_eq!(content.as_bytes(), decrypted.as_slice());
        }
    }
}
