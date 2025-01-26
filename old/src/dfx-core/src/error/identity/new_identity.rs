use crate::error::identity::load_pem::LoadPemError;
use crate::error::identity::load_pem_identity::LoadPemIdentityError;
use crate::error::identity::new_hardware_identity::NewHardwareIdentityError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NewIdentityError {
    #[error("Failed to load PEM")]
    LoadPemFailed(#[source] LoadPemError),

    #[error("Failed to load PEM identity")]
    LoadPemIdentityFailed(#[source] LoadPemIdentityError),

    #[error("Failed to instantiate hardware identity")]
    NewHardwareIdentityFailed(#[source] NewHardwareIdentityError),
}
