use crate::lib::error::DfxResult;
use crate::Environment;
use dfx_core::network::root_key::fetch_root_key_if_needed;

use anyhow::{anyhow, Context};
use candid::Principal;
use clap::Parser;
use std::io::{stdout, Write};
use tokio::runtime::Runtime;

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
    let runtime = Runtime::new().expect("Unable to create a runtime");

    let callee_canister = opts.canister_name.as_str();
    let canister_id_store = env.get_canister_id_store()?;

    let canister_id = Principal::from_text(callee_canister)
        .or_else(|_| canister_id_store.get(callee_canister))?;

    let network = env.get_network_descriptor();
    runtime.block_on(async { fetch_root_key_if_needed(&agent, &network).await })?;

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
