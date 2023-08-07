use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::state_tree::canister_info::{
    read_state_tree_canister_controllers, read_state_tree_canister_module_hash,
};
use anyhow::anyhow;
use candid::Principal;
use clap::Parser;
use itertools::Itertools;

/// Get the hash of a canister’s WASM module and its current controllers.
#[derive(Parser)]
pub struct InfoOpts {
    /// Specifies the name or id of the canister to get its canister information.
    canister: String,
}

pub async fn exec(env: &dyn Environment, opts: InfoOpts) -> DfxResult {
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    let callee_canister = opts.canister.as_str();
    let canister_id_store = env.get_canister_id_store()?;

    let canister_id = Principal::from_text(callee_canister)
        .or_else(|_| canister_id_store.get(callee_canister))?;

    fetch_root_key_if_needed(env).await?;

    let controllers_sorted: Vec<_> = read_state_tree_canister_controllers(agent, canister_id)
        .await?
        .ok_or_else(|| anyhow!("Canister {canister_id} does not exist."))?
        .iter()
        .map(Principal::to_text)
        .sorted()
        .collect();

    let module_hash_hex = match read_state_tree_canister_module_hash(agent, canister_id).await? {
        None => "None".to_string(),
        Some(blob) => format!("0x{}", hex::encode(blob)),
    };

    println!(
        "Controllers: {}\nModule hash: {}",
        controllers_sorted.join(" "),
        module_hash_hex
    );

    Ok(())
}
