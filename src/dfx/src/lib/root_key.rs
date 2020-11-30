use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use anyhow::anyhow;

pub async fn fetch_root_key_if_needed(env: &dyn Environment) -> DfxResult {
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    agent.fetch_root_key().await?;
    Ok(())
}
