use crate::lib::error::DfxResult;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::Environment;

use anyhow::{anyhow, Context};
use candid::Principal;
use clap::Parser;
use std::io::{stdout, Write};

/// Displays metadata in a canister.
#[derive(Parser)]
pub struct CanisterMetadataOpts {
    /// Specifies the name of the canister to call.
    canister_name: String,

    /// Specifies the name of the metadata to retrieve.
    metadata_name: String,
}

pub async fn exec(env: &dyn Environment, opts: CanisterMetadataOpts) -> DfxResult {
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    let callee_canister = opts.canister_name.as_str();
    let canister_id_store = CanisterIdStore::for_env(env)?;

    let canister_id = Principal::from_text(callee_canister)
        .or_else(|_| canister_id_store.get(callee_canister))?;

    fetch_root_key_if_needed(env).await?;
    let metadata = agent
        .read_state_canister_metadata(canister_id, &opts.metadata_name, false)
        .await
        .with_context(|| format!("Failed to read controllers of canister {}.", canister_id))?;

    stdout().write_all(&metadata)?;

    Ok(())
}
