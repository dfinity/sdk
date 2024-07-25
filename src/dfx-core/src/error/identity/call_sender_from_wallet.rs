use crate::error::wallet_config::WalletConfigError;
use candid::types::principal::PrincipalError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CallSenderFromWalletError {
    #[error("Failed to read principal from id '{0}', and did not find a wallet for that identity")]
    ParsePrincipalFromIdFailedAndNoWallet(String, #[source] PrincipalError),

    #[error("Failed to read principal from id '{0}' ({1}), and failed to load the wallet for that identity"
    )]
    ParsePrincipalFromIdFailedAndGetWalletCanisterIdFailed(
        String,
        PrincipalError,
        #[source] WalletConfigError,
    ),
}
