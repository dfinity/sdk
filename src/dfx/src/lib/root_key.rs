use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use anyhow::anyhow;

pub async fn fetch_root_key_if_needed<'a>(env: &'a (dyn Environment + 'a)) -> DfxResult {
    let must_fetch_root_key = env
        .get_network_descriptor()
        .map(|nd| nd.name != "ic")
        .unwrap_or(true);

    if must_fetch_root_key {
        let agent = env
            .get_agent()
            .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

        agent.fetch_root_key().await?;
    }
    Ok(())
}
