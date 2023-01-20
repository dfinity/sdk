use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use dfx_core::error::identity::IdentityError;
use dfx_core::error::identity::IdentityError::{UnsupportedKeyVersion, ValidatePemContentFailed};

use anyhow::Context;
use candid::Principal;
use fn_error_context::context;
use ic_agent::identity::BasicIdentity;
use ic_agent::identity::PemError;
use ic_agent::identity::Secp256k1Identity;

#[derive(Debug, PartialEq, Eq)]
pub enum CallSender {
    SelectedId,
    Wallet(Principal),
}

// Determine whether the selected Identity
// or the provided wallet canister ID should be the Sender of the call.
#[context("Failed to determine call sender.")]
pub async fn call_sender(_env: &dyn Environment, wallet: &Option<String>) -> DfxResult<CallSender> {
    let sender = if let Some(id) = wallet {
        CallSender::Wallet(
            Principal::from_text(id)
                .with_context(|| format!("Failed to read principal from {:?}.", id))?,
        )
    } else {
        CallSender::SelectedId
    };
    Ok(sender)
}

pub fn validate_pem_file(pem_content: &[u8]) -> Result<(), IdentityError> {
    let secp_res =
        Secp256k1Identity::from_pem(pem_content).map_err(|e| ValidatePemContentFailed(Box::new(e)));
    if let Err(e) = secp_res {
        let basic_identity_res = BasicIdentity::from_pem(pem_content);
        match basic_identity_res {
            Err(PemError::KeyRejected(rj)) if rj.description_() == "VersionNotSupported" => {
                return Err(UnsupportedKeyVersion());
            }
            Err(_) => return Err(e),
            _ => {}
        }
    }

    Ok(())
}
