use thiserror::Error;

#[derive(Error, Debug)]
pub enum EncryptionError {
    #[error("Failed to decrypt content")]
    DecryptContentFailed(#[source] aes_gcm::Error),

    #[error("Failed to encrypt content")]
    EncryptContentFailed(#[source] aes_gcm::Error),

    #[error("Failed to hash password")]
    HashPasswordFailed(#[source] argon2::password_hash::Error),

    #[error("Failed to generate nonce")]
    NonceGenerationFailed(#[source] ring::error::Unspecified),

    #[error("Failed to read user input")]
    ReadUserPasswordFailed(#[source] dialoguer::Error),

    #[error("Failed to generate salt")]
    SaltGenerationFailed(#[source] ring::error::Unspecified),
}
