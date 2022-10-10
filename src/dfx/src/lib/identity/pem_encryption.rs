use std::path::Path;

use crate::lib::error::DfxResult;

use super::identity_manager::EncryptionConfiguration;
use super::IdentityConfiguration;

use crate::lib::identity::pem_encryption::PromptMode::{DecryptingToUse, EncryptingToCreate};
use aes_gcm::aead::{Aead, NewAead};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use anyhow::{anyhow, Context};
use argon2::{password_hash::PasswordHasher, Argon2};
use fn_error_context::context;

/// Transparently handles all complexities regarding pem file encryption, including prompting the user for the password.
///
/// Try to only load the pem file once, as the user may be prompted for the password every single time you call this function.
#[context("Failed to load pem file {}.", path.to_string_lossy())]
pub fn load_pem_file(path: &Path, config: Option<&IdentityConfiguration>) -> DfxResult<Vec<u8>> {
    let content = std::fs::read(path)
        .with_context(|| format!("Failed to read {}.", path.to_string_lossy()))?;
    let content = maybe_decrypt_pem(content.as_slice(), config)?;
    Ok(content)
}

/// Transparently handles all complexities regarding pem file encryption, including prompting the user for the password.
///
/// Automatically creates required directories.
#[context("Failed to write pem file.")]
pub fn write_pem_file(
    path: &Path,
    config: Option<&IdentityConfiguration>,
    pem_content: &[u8],
) -> DfxResult<()> {
    let pem_content = maybe_encrypt_pem(pem_content, config)?;

    let containing_folder = path.parent().with_context(|| {
        format!(
            "Could not determine parent folder for {}",
            path.to_string_lossy()
        )
    })?;
    std::fs::create_dir_all(containing_folder)
        .with_context(|| format!("Failed to create {}.", containing_folder.to_string_lossy()))?;
    std::fs::write(path, pem_content)
        .with_context(|| format!("Failed to write pem file to {}.", path.to_string_lossy()))?;

    let mut permissions = std::fs::metadata(path)
        .with_context(|| format!("Failed to read permissions of {}.", path.to_string_lossy()))?
        .permissions();
    permissions.set_readonly(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        permissions.set_mode(0o400);
    }
    std::fs::set_permissions(path, permissions)
        .with_context(|| format!("Failed to set permissions of {}.", path.to_string_lossy()))?;

    Ok(())
}

/// If the IndentityConfiguration suggests that the content of the pem file should be encrypted,
/// then the user is prompted for the password to the pem file.
/// The encrypted pem file content is then returned.
///
/// If the pem file should not be encrypted, then the content is returned as is.
///
/// `maybe_decrypt_pem` does the opposite.
#[context("Failed to encrypt pem file.")]
fn maybe_encrypt_pem(
    pem_content: &[u8],
    config: Option<&IdentityConfiguration>,
) -> DfxResult<Vec<u8>> {
    if let Some(encryption_config) = config.and_then(|c| c.encryption.as_ref()) {
        let password = password_prompt(EncryptingToCreate)?;
        let result = encrypt(pem_content, encryption_config, &password);
        println!("Encryption complete.");
        result
    } else {
        Ok(Vec::from(pem_content))
    }
}

/// If the IndentityConfiguration suggests that the content of the pem file is encrypted,
/// then the user is prompted for the password to the pem file.
/// The decrypted pem file content is then returned.
///
/// If the pem file should not be encrypted, then the content is returned as is.
///
/// `maybe_encrypt_pem` does the opposite.
#[context("Failed to decrypt pem file.")]
fn maybe_decrypt_pem(
    pem_content: &[u8],
    config: Option<&IdentityConfiguration>,
) -> DfxResult<Vec<u8>> {
    if let Some(decryption_config) = config.and_then(|c| c.encryption.as_ref()) {
        let password = password_prompt(DecryptingToUse)?;
        let result = decrypt(pem_content, decryption_config, &password);
        if result.is_ok() {
            // print to stderr so that output redirection works for the identity export command
            eprintln!("Decryption complete.");
        };
        result
    } else {
        Ok(Vec::from(pem_content))
    }
}

enum PromptMode {
    EncryptingToCreate,
    DecryptingToUse,
}

#[context("Failed to prompt user for password.")]
fn password_prompt(mode: PromptMode) -> DfxResult<String> {
    let prompt = match mode {
        PromptMode::EncryptingToCreate => "Please enter a passphrase for your identity",
        PromptMode::DecryptingToUse => "Please enter the passphrase for your identity",
    };
    let pw = dialoguer::Password::new()
        .with_prompt(prompt)
        .interact()
        .context("Failed to read user input.")?;
    Ok(pw)
}

fn get_argon_params() -> argon2::Params {
    argon2::Params::new(64000 /* in kb */, 3, 1, Some(32 /* in bytes */)).unwrap()
}

#[context("Failed during encryption.")]
fn encrypt(content: &[u8], config: &EncryptionConfiguration, password: &str) -> DfxResult<Vec<u8>> {
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
        .map_err(|e| anyhow!("Failed to encrypt content: {}", e))?;

    Ok(encrypted)
}

#[context("Failed during decryption.")]
fn decrypt(
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
        .map_err(|e| anyhow!("Failed to decrypt content: {}.", e))?;
    Ok(decrypted)
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(90))] // takes ~0.3s per case
        #[test]
        fn decrypt_reverts_encrypt(pass in ".*", content in ".*") {
            let config = EncryptionConfiguration::new().unwrap();
            let encrypted = encrypt(content.as_bytes(), &config, &pass).unwrap();
            let decrypted = decrypt(&encrypted, &config, &pass).unwrap();

            assert_eq!(content.as_bytes(), decrypted.as_slice());
        }
    }
}
