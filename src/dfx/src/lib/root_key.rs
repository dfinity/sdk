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
            .timeout(std::time::Duration::from_secs(30))
            .throttle(std::time::Duration::from_secs(1))
            .build();
        waiter.start();

        loop {

            let fetch_result = agent.fetch_root_key().await;
            if fetch_result.is_ok() {
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
