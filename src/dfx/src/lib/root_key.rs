use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use anyhow::anyhow;
use garcon::{Delay, Waiter};

pub async fn fetch_root_key_if_needed(env: &dyn Environment) -> DfxResult {
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    if !env
        .get_network_descriptor()
        .expect("no network descriptor")
        .is_ic
    {
        let mut waiter = Delay::builder()
            .exponential_backoff(std::time::Duration::from_secs(1), 1.1)
            .timeout(std::time::Duration::from_secs(60 * 5))
            .build();
        waiter.start();

        loop {
            match agent.fetch_root_key().await {
                Ok(()) => return Ok(()),
                Err(fetch_err) => waiter.wait().map_err(|_| fetch_err),
            }?;
        }
    }
    Ok(())
}
