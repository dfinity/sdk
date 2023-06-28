use thiserror::Error;

#[derive(Error, Debug)]
pub enum UserConsent {
    #[error("Unable to read input: {0}")]
    ReadError(std::io::Error),

    #[error("User declined consent.")]
    Declined,
}
