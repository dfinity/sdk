use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use anyhow::{anyhow, Context};
use fn_error_context::context;

#[context("Failed to fetch root key.")]
pub async fn fetch_root_key_if_needed(env: &dyn Environment) -> DfxResult {
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    if !env.get_network_descriptor().is_ic {
        agent
            .fetch_root_key()
            .await
            .context("Encountered an error while trying to query the replica.")?;
    }
    Ok(())
}

/// Fetches the root key of the local network.
/// Returns an error if attempted to run on the real IC.
#[context("Failed to fetch root key.")]
pub async fn fetch_root_key_or_anyhow(env: &dyn Environment) -> DfxResult {
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    if !env.get_network_descriptor().is_ic {
        agent
            .fetch_root_key()
            .await
            .context("Encountered an error while trying to query the local replica.")?;
        Ok(())
    } else {
        Err(anyhow!(
            "This command only runs on local instances. Cannot run this on the real IC."
        ))
    }
}
