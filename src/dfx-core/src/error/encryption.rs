use thiserror::Error;

#[derive(Error, Debug)]
pub enum EncryptionError {
    #[error("Failed to encrypt content: {0}")]
    EncryptContentFailed(aes_gcm::Error),

    #[error("Failed to hash password: {0}")]
    HashPasswordFailed(argon2::password_hash::Error),

    #[error("Failed to read user input: {0}")]
    ReadUserPasswordFailed(std::io::Error),
}
