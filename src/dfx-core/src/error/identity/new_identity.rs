use crate::error::identity::load_pem::LoadPemError;
use crate::error::identity::load_pem_identity::LoadPemIdentityError;
use crate::error::identity::new_hardware_identity::NewHardwareIdentityError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NewIdentityError {
    #[error("Failed to load PEM: {0}")]
    LoadPemFailed(LoadPemError),

    #[error("Failed to load PEM identity: {0}")]
    LoadPemIdentityFailed(LoadPemIdentityError),

    #[error("Failed to instantiate hardware identity: {0}")]
    NewHardwareIdentityFailed(NewHardwareIdentityError),

    #[error("There was some issue with the identity creation")]
    DelegatedIdentityCreationFailed,
}
