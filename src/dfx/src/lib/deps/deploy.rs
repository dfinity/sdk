use crate::lib::error::DfxResult;
use crate::lib::state_tree::canister_info::read_state_tree_canister_controllers;

use anyhow::bail;
use candid::Principal;
use fn_error_context::context;
use ic_agent::Agent;
use ic_utils::interfaces::ManagementCanister;
use slog::{info, Logger};

use super::{get_canister_prompt, PulledCanister};

// not use operations::canister::create_canister because we don't want to modify canister_id_store
#[context("Failed to create canister {}", canister_id)]
pub async fn try_create_canister(
    agent: &Agent,
    logger: &Logger,
    canister_id: &Principal,
    pulled_canister: &PulledCanister,
) -> DfxResult {
    let canister_prompt = get_canister_prompt(canister_id, pulled_canister);
    match read_state_tree_canister_controllers(agent, *canister_id).await? {
        Some(cs) if cs.len() == 1 && cs[0] == Principal::anonymous() => Ok(()),
        Some(_) => {
            bail!("Canister {canister_id} has been created before and its controller is not the anonymous identity. Please stop and delete it and then deploy again.");
        }
        None => {
            let mgr = ManagementCanister::create(agent);
            info!(logger, "Creating canister: {canister_prompt}");
            mgr.create_canister()
                .as_provisional_create_with_specified_id(*canister_id)
                .call_and_wait()
                .await?;
            Ok(())
        }
    }
}
