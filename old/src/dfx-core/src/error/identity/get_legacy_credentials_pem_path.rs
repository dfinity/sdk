use crate::error::get_user_home::GetUserHomeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GetLegacyCredentialsPemPathError {
    #[error("Failed to get legacy pem path")]
    GetLegacyPemPathFailed(#[source] GetUserHomeError),
}
