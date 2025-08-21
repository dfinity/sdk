use super::{PulledCanister, get_canister_prompt};
use crate::lib::error::DfxResult;
use crate::lib::state_tree::canister_info::read_state_tree_canister_controllers;
use anyhow::bail;
use candid::Principal;
use fn_error_context::context;
use ic_agent::Agent;
use ic_utils::interfaces::ManagementCanister;
use slog::{Logger, info};

// not use operations::canister::create_canister because we don't want to modify canister_id_store
#[context("Failed to create canister {}", canister_id)]
pub async fn try_create_canister(
    agent: &Agent,
    logger: &Logger,
    canister_id: &Principal,
    pulled_canister: &PulledCanister,
) -> DfxResult {
    let canister_prompt = get_canister_prompt(canister_id, pulled_canister);
    // Check if the canister has been created before.
    // If read_state_tree_canister_controllers returns Err, then the pocket-ic doesn't have a subnet covering the canister_id yet.
    // We can safely create the canister in this case because the pocket-ic will automatically create the subnet when the canister is created.
    if let Ok(Some(cs)) = read_state_tree_canister_controllers(agent, *canister_id).await {
        if cs.len() == 1 && cs[0] == Principal::anonymous() {
            return Ok(());
        } else {
            bail!(
                "Canister {canister_id} has been created before and its controller is not the anonymous identity. Please stop and delete it and then deploy again."
            );
        }
    }

    let mgr = ManagementCanister::create(agent);
    info!(logger, "Creating canister: {canister_prompt}");
    mgr.create_canister()
        .as_provisional_create_with_specified_id(*canister_id)
        .await?;
    Ok(())
}
