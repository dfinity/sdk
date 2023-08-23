use crate::error::foundation::FoundationError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GetLegacyCredentialsPemPathError {
    #[error("Failed to get legacy pem path: {0}")]
    GetLegacyPemPathFailed(FoundationError),
}
