use ic_identity_hsm::HardwareIdentityError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NewHardwareIdentityError {
    #[error("Failed to instantiate hardware identity for identity '{0}': {1}.")]
    InstantiateHardwareIdentityFailed(String, Box<HardwareIdentityError>),
}
