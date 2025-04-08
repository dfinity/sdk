use crate::lib::error::DfxResult;
use anyhow::{anyhow, bail, Context};
use candid::Principal;
use ic_agent::{Agent, AgentError};

pub async fn read_state_tree_canister_controllers(
    agent: &Agent,
    canister_id: Principal,
) -> DfxResult<Option<Vec<Principal>>> {
    let controllers = match agent.read_state_canister_controllers(canister_id).await {
        Err(AgentError::LookupPathAbsent(_)) => {
            return Ok(None);
        }
        Err(AgentError::InvalidCborData(_)) => {
            return Err(anyhow!("Invalid cbor data in controllers canister info.").into());
        }
        r => r.with_context(|| format!("Failed to read controllers of canister {canister_id}."))?,
    };

    Ok(Some(controllers))
}

/// None can indicate either of these, but we can't tell from here:
/// - the canister doesn't exist
/// - the canister exists but does not have a module installed
pub async fn read_state_tree_canister_module_hash(
    agent: &Agent,
    canister_id: Principal,
) -> DfxResult<Option<Vec<u8>>> {
    let module_hash = match agent.read_state_canister_module_hash(canister_id).await {
        Ok(blob) => Some(blob),
        Err(AgentError::LookupPathAbsent(_)) => None,
        Err(x) => bail!(x),
    };

    Ok(module_hash)
}
