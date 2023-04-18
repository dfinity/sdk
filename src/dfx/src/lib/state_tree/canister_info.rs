use crate::lib::error::DfxResult;

use anyhow::{anyhow, bail, Context};
use candid::Principal;
use ic_agent::{Agent, AgentError};
use serde_cbor::Value;

pub async fn read_state_tree_canister_controllers(
    agent: &Agent,
    canister_id: Principal,
) -> DfxResult<Option<Vec<Principal>>> {
    let controller_blob = match agent
        .read_state_canister_info(canister_id, "controllers")
        .await
    {
        Err(AgentError::LookupPathUnknown(_) | AgentError::LookupPathAbsent(_)) => {
            return Ok(None);
        }
        r => r.with_context(|| format!("Failed to read controllers of canister {canister_id}."))?,
    };
    let cbor: Value = serde_cbor::from_slice(&controller_blob)
        .map_err(|_| anyhow!("Invalid cbor data in controllers canister info."))?;
    let controllers = if let Value::Array(vec) = cbor {
        vec.into_iter()
            .map(|elem: Value| {
                if let Value::Bytes(bytes) = elem {
                    Ok(Principal::try_from(&bytes).with_context(|| {
                        format!(
                            "Failed to construct principal of controller from bytes ({}).",
                            hex::encode(&bytes)
                        )
                    })?)
                } else {
                    bail!(
                        "Expected element in controllers to be of type bytes, got {:?}",
                        elem
                    );
                }
            })
            .collect::<DfxResult<Vec<Principal>>>()
    } else {
        bail!("Expected controllers to be an array, but got {:?}", cbor);
    }
    .context("Failed to determine controllers.")?;

    Ok(Some(controllers))
}

/// None can indicate either of these, but we can't tell from here:
/// - the canister doesn't exist
/// - the canister exists but does not have a module installed
pub async fn read_state_tree_canister_module_hash(
    agent: &Agent,
    canister_id: Principal,
) -> DfxResult<Option<Vec<u8>>> {
    let module_hash = match agent
        .read_state_canister_info(canister_id, "module_hash")
        .await
    {
        Ok(blob) => Some(blob),
        // If the canister is empty, this path does not exist.
        // The replica doesn't support negative lookups, therefore if the canister
        // is empty, the replica will return lookup_path([], Pruned _) = Unknown
        Err(AgentError::LookupPathUnknown(_)) | Err(AgentError::LookupPathAbsent(_)) => None,
        Err(x) => bail!(x),
    };

    Ok(module_hash)
}
