use crate::error::identity::require_identity_exists::RequireIdentityExistsError;
use crate::error::identity::write_default_identity::WriteDefaultIdentityError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UseIdentityByNameError {
    #[error("Identity must exist")]
    RequireIdentityExistsFailed(#[source] RequireIdentityExistsError),

    #[error("Failed to write default identity")]
    WriteDefaultIdentityFailed(#[source] WriteDefaultIdentityError),
}
