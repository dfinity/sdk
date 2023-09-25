use crate::error::identity::get_identity_config_or_default::GetIdentityConfigOrDefaultError;
use crate::error::identity::new_identity::NewIdentityError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadIdentityError {
    #[error("Failed to get identity config: {0}")]
    GetIdentityConfigOrDefaultFailed(GetIdentityConfigOrDefaultError),

    #[error("Failed to instantiate identity: {0}")]
    NewIdentityFailed(NewIdentityError),
}
