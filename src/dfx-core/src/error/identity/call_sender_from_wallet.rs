use candid::types::principal::PrincipalError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CallSenderFromWalletError {
    #[error("Failed to read principal from id '{0}'")]
    ParsePrincipalFromIdFailed(String, #[source] PrincipalError),
}
