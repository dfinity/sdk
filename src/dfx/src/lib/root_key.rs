use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use anyhow::{anyhow, Context};

pub async fn fetch_root_key_if_needed(env: &dyn Environment) -> DfxResult {
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    if !env
        .get_network_descriptor()
        .expect("no network descriptor")
        .is_ic
    {
        agent
            .fetch_root_key()
            .await
            .context("Failed during call to replica.")?;
    }
    Ok(())
}

/// Fetches the root key of the local network.
/// Returns an error if attempted to run on the real IC.
pub async fn fetch_root_key_or_anyhow(env: &dyn Environment) -> DfxResult {
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    if !env
        .get_network_descriptor()
        .expect("no network descriptor")
        .is_ic
    {
        agent
            .fetch_root_key()
            .await
            .context("Failed during call to replica.")?;
        Ok(())
    } else {
        Err(anyhow!(
            "This command only runs on local instances. Cannot run this on the real IC."
        ))
    }
}
