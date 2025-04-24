use crate::lib::error::DfxResult;
use anyhow::Context;
use candid::Principal;
use ic_agent::Agent;

pub async fn read_state_tree_canister_controllers(
    agent: &Agent,
    canister_id: Principal,
) -> DfxResult<Option<Vec<Principal>>> {
    agent
        .read_state_canister_controllers(canister_id)
        .await
        .with_context(|| format!("Failed to read controllers of canister {canister_id}."))
}

/// None can indicate either of these, but we can't tell from here:
/// - the canister doesn't exist
/// - the canister exists but does not have a module installed
pub async fn read_state_tree_canister_module_hash(
    agent: &Agent,
    canister_id: Principal,
) -> DfxResult<Option<Vec<u8>>> {
    Ok(agent.read_state_canister_module_hash(canister_id).await?)
}
