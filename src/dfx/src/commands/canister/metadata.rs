use crate::lib::error::DfxResult;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::Environment;
use anyhow::Context;
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
    let agent = env.get_agent();

    let callee_canister = opts.canister_name.as_str();
    let canister_id_store = env.get_canister_id_store()?;

    let canister_id = Principal::from_text(callee_canister)
        .or_else(|_| canister_id_store.get(callee_canister))?;

    fetch_root_key_if_needed(env).await?;
    let metadata = agent
        .read_state_canister_metadata(canister_id, &opts.metadata_name)
        .await
        .with_context(|| {
            format!(
                "Failed to read `{}` metadata of canister {}.",
                opts.metadata_name, canister_id
            )
        })?;

    stdout().write_all(&metadata)?;

    Ok(())
}
