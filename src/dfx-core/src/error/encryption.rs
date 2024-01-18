use thiserror::Error;

#[derive(Error, Debug)]
pub enum EncryptionError {
    #[error("Failed to decrypt content: {0}")]
    DecryptContentFailed(aes_gcm::Error),

    #[error("Failed to encrypt content: {0}")]
    EncryptContentFailed(aes_gcm::Error),

    #[error("Failed to hash password: {0}")]
    HashPasswordFailed(argon2::password_hash::Error),

    #[error("Failed to generate nonce: {0}")]
    NonceGenerationFailed(ring::error::Unspecified),

    #[error("Failed to read user input: {0}")]
    ReadUserPasswordFailed(dialoguer::Error),

    #[error("Failed to generate salt: {0}")]
    SaltGenerationFailed(ring::error::Unspecified),
}
