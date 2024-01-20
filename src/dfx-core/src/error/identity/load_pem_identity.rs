use ic_agent::identity::PemError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadPemIdentityError {
    #[error("Cannot read identity file '{0}': {1:#}")]
    ReadIdentityFileFailed(String, Box<PemError>),
}
