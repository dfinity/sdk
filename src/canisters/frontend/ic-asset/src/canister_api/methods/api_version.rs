use crate::canister_api::methods::method_names::API_VERSION;
use ic_agent::AgentError;
use ic_utils::Canister;
use ic_utils::call::SyncCall;

const CANISTER_METHOD_NOT_FOUND: &str = "IC0536";

fn is_method_not_found(err: &AgentError) -> bool {
    match err {
        AgentError::CertifiedReject { reject, .. }
        | AgentError::UncertifiedReject { reject, .. } => {
            reject.error_code.as_deref() == Some(CANISTER_METHOD_NOT_FOUND)
        }
        _ => false,
    }
}

pub(crate) async fn api_version(canister: &Canister<'_>) -> Result<u16, AgentError> {
    match canister.query(API_VERSION).build().call().await {
        Ok((version,)) => Ok(version),
        // If the canister doesn't have the `api_version` method, it's an old version of the API.
        Err(e) if is_method_not_found(&e) => Ok(0),
        Err(e) => Err(e),
    }
}
