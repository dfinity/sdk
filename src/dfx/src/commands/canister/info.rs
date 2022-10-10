use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::root_key::fetch_root_key_if_needed;

use anyhow::{anyhow, bail, Context};
use candid::Principal;
use clap::Parser;
use ic_agent::AgentError;
use serde_cbor::Value;
use std::convert::TryFrom;

/// Get the hash of a canister’s WASM module and its current controller.
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
    let canister_id_store = CanisterIdStore::for_env(env)?;

    let canister_id = Principal::from_text(callee_canister)
        .or_else(|_| canister_id_store.get(callee_canister))?;

    fetch_root_key_if_needed(env).await?;
    let controller_blob = match agent
        .read_state_canister_info(canister_id, "controllers", false)
        .await
    {
        Err(AgentError::LookupPathUnknown(_) | AgentError::LookupPathAbsent(_)) => {
            bail!("Canister {canister_id} does not exist.")
        }
        r => r.with_context(|| format!("Failed to read controllers of canister {canister_id}."))?,
    };
    let cbor: Value = serde_cbor::from_slice(&controller_blob)
        .map_err(|_| anyhow!("Invalid cbor data in controllers canister info."))?;
    let controllers = if let Value::Array(vec) = cbor {
        vec.into_iter()
            .map(|elem: Value| {
                if let Value::Bytes(bytes) = elem {
                    Ok(Principal::try_from(&bytes)
                        .with_context(|| {
                            format!(
                                "Failed to construct principal of controller from bytes ({}).",
                                hex::encode(&bytes)
                            )
                        })?
                        .to_text())
                } else {
                    bail!(
                        "Expected element in controllers to be of type bytes, got {:?}",
                        elem
                    );
                }
            })
            .collect::<DfxResult<Vec<String>>>()
    } else {
        bail!("Expected controllers to be an array, but got {:?}", cbor);
    }
    .context("Failed to determine controllers.")?;

    let module_hash_hex = match agent
        .read_state_canister_info(canister_id, "module_hash", false)
        .await
    {
        Ok(blob) => format!("0x{}", hex::encode(&blob)),
        // If the canister is empty, this path does not exist.
        // The replica doesn't support negative lookups, therefore if the canister
        // is empty, the replica will return lookup_path([], Pruned _) = Unknown
        Err(AgentError::LookupPathUnknown(_)) | Err(AgentError::LookupPathAbsent(_)) => {
            "None".to_string()
        }
        Err(x) => bail!(x),
    };

    let mut controllers_sorted = controllers;
    controllers_sorted.sort();

    println!(
        "Controllers: {}\nModule hash: {}",
        controllers_sorted.join(" "),
        module_hash_hex
    );

    Ok(())
}
