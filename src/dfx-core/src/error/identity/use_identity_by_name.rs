use crate::error::identity::write_default_identity::WriteDefaultIdentityError;
use crate::error::identity::IdentityError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UseIdentityByNameError {
    #[error("Identity must exist: {0}")]
    RequireIdentityExistsFailed(IdentityError),

    #[error("Failed to write default identity: {0}")]
    WriteDefaultIdentityFailed(WriteDefaultIdentityError),
}
