use thiserror::Error;

#[derive(Error, Debug)]
pub enum UserConsent {
    #[error("Unable to read input")]
    ReadError(#[source] std::io::Error),

    #[error("User declined consent.")]
    Declined,
}
