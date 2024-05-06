use crate::lib::environment::{create_agent, Environment};
use crate::lib::error::{DfxError, DfxResult};
use anyhow::{bail, Context};
use clap::Parser;
use dfx_core::identity::Identity;
use dfx_core::network::provider::{
    command_line_provider_to_url, create_network_descriptor, get_network_context,
    LocalBindDetermination,
};
use dfx_core::util::expiry_duration;
use slog::warn;
use std::time::Duration;
use tokio::runtime::Runtime;

/// Pings an Internet Computer network and returns its status.
#[derive(Parser)]
pub struct PingOpts {
    /// The provider to use.
    /// A valid URL (starting with `http:` or `https:`) can be used here, and a special
    /// ephemeral network will be created specifically for this request. E.g.
    /// "http://localhost:12345/" is a valid network name.
    network: Option<String>,

    /// Repeatedly ping until the replica is healthy
    #[arg(long)]
    wait_healthy: bool,
}

pub fn exec(env: &dyn Environment, opts: PingOpts) -> DfxResult {
    // For ping, "provider" could either be a URL or a network name.
    // If not passed, we default to the "local" network.
    let agent_url = create_network_descriptor(
        env.get_config()?,
        env.get_networks_config(),
        opts.network,
        None,
        LocalBindDetermination::ApplyRunningWebserverPort,
    )
    .and_then(|network_descriptor| {
        let url = network_descriptor.first_provider()?.to_string();
        Ok(url)
    })
    .or_else::<DfxError, _>(|err| {
        let logger = env.get_logger();
        warn!(logger, "{:#}", err);
        let network_name = get_network_context()?;
        let url = command_line_provider_to_url(&network_name)?;
        Ok(url)
    })?;

    let timeout = expiry_duration();
    let identity = Box::new(Identity::anonymous());
    let agent = create_agent(env.get_logger().clone(), &agent_url, identity, timeout)?;

    let runtime = Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(async {
        if opts.wait_healthy {
            let mut retries = 0;

            loop {
                let status = agent.status().await;
                if let Ok(status) = status {
                    let healthy = match &status.replica_health_status {
                        Some(s) if s == "healthy" => true,
                        None => true,
                        _ => false,
                    };
                    if healthy {
                        println!("{}", status);
                        break;
                    } else {
                        eprintln!("{}", status);
                    }
                }
                if retries >= 60 {
                    bail!("Timed out waiting for replica to become healthy");
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
                retries += 1;
            }
        } else {
            let status = agent
                .status()
                .await
                .context("Failed while waiting for agent status.")?;
            println!("{}", status);
        }

        Ok(())
    })
}
