use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use anyhow::anyhow;

pub async fn fetch_api_version(env: &dyn Environment) -> DfxResult<String> {
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    let status = agent.status().await?;
    Ok(status.ic_api_version)
}
