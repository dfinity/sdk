use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::root_key::fetch_root_key_if_needed;

use anyhow::{anyhow, bail};
use clap::Clap;
use ic_agent::AgentError;
use ic_types::Principal;
use std::convert::TryFrom;

/// Get the hash of a canister’s WASM module and its current controller in a certified way.
#[derive(Clap)]
pub struct ReadStateOpts {
    /// Specifies the name or id of the canister to get its certified canister information.
    canister_name: String,
}

pub async fn exec(env: &dyn Environment, opts: ReadStateOpts) -> DfxResult {
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    let callee_canister = opts.canister_name.as_str();
    let canister_id_store = CanisterIdStore::for_env(env)?;

    let canister_id = Principal::from_text(callee_canister)
        .or_else(|_| canister_id_store.get(callee_canister))?;

    fetch_root_key_if_needed(env).await?;
    let controller_blob = agent
        .read_state_canister_info(canister_id.clone(), "controller")
        .await?;
    let controller = Principal::try_from(controller_blob)?.to_text();
    let module_hash_hex = match agent
        .read_state_canister_info(canister_id, "module_hash")
        .await
    {
        Ok(blob) => format!("0x{}", hex::encode(&blob)),
        // If the canister is empty, this path does not exist.
        Err(AgentError::LookupPathUnknown(_)) => "None".to_string(),
        Err(x) => bail!(x),
    };

    println!(
        "Controller: {}\nModule hash: {}",
        controller, module_hash_hex
    );

    Ok(())
}
