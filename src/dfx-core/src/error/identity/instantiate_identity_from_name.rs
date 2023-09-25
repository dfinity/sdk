use crate::error::identity::load_identity::LoadIdentityError;
use crate::error::identity::require_identity_exists::RequireIdentityExistsError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InstantiateIdentityFromNameError {
    #[error("Failed to get principal of identity: {0}")]
    GetIdentityPrincipalFailed(String),

    #[error("Failed to load identity: {0}")]
    LoadIdentityFailed(LoadIdentityError),

    #[error("Identity must exist: {0}")]
    RequireIdentityExistsFailed(RequireIdentityExistsError),
}
