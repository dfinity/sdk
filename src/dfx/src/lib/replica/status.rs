use crate::lib::error::DfxResult;

use anyhow::{bail, Context};
use ic_agent::agent::http_transport::ReqwestHttpReplicaV2Transport;
use ic_agent::Agent;
use std::time::Duration;

pub async fn ping_and_wait(url: &str) -> DfxResult {
    let agent = Agent::builder()
        .with_transport(
            ReqwestHttpReplicaV2Transport::create(url)
                .with_context(|| format!("Failed to create replica transport from url {url}.",))?,
        )
        .build()
        .with_context(|| format!("Failed to build agent with url {url}."))?;
    let mut retries = 0;
    loop {
        let status = agent.status().await;
        match status {
            Ok(status) => {
                let healthy = match &status.replica_health_status {
                    Some(status) if status == "healthy" => true,
                    None => true, // emulator doesn't report replica_health_status
                    _ => false,
                };
                if healthy {
                    break;
                }
            }
            Err(e) => {
                if retries >= 60 {
                    bail!(e);
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
                retries += 1;
            }
        }
    }
    Ok(())
}
