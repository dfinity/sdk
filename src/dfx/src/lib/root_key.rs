use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use anyhow::{anyhow, bail};
use garcon::{Delay, Waiter};
use ic_agent::agent::status::Value;
use ic_agent::Agent;

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
            .exponential_backoff(std::time::Duration::from_secs(1), 2.0)
            .timeout(std::time::Duration::from_secs(60 * 5))
            .build();
        // let mut waiter = Delay::builder()
        //     .timeout(std::time::Duration::from_secs(30))
        //     .throttle(std::time::Duration::from_secs(1))
        //     .build();
        waiter.start();

        loop {
            let fetch_result = agent.fetch_root_key().await;
            if fetch_result.is_ok() {
                wait_for_status_healthy(agent).await?;
                return Ok(());
            };

            let wait_result = waiter.wait();
            if wait_result.is_err() {
                fetch_result?;
            };
        }
    }
    Ok(())
}

async fn wait_for_status_healthy(agent: &Agent) -> DfxResult {
    let mut waiter = Delay::builder()
        .exponential_backoff(std::time::Duration::from_secs(1), 2.0)
        .timeout(std::time::Duration::from_secs(60 * 5))
        .build();
    waiter.start();

    loop {
        if let Ok(status) = agent.status().await {
            if let Some(v) = status.values.get("replica_health_status") {
                if let Value::String(s) = v.as_ref() {
                    let s = s.to_owned();
                    if s == "healthy" {
                        return Ok(());
                    }
                }
            }
        }
        if waiter.wait().is_err() {
            bail!("replica did not become healthy in time");
        }
    }
}
