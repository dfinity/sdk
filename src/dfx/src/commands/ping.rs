use crate::config::dfinity::{NetworkType, DEFAULT_IC_GATEWAY};
use crate::lib::environment::{AgentEnvironment, Environment};
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::network::network_descriptor::NetworkDescriptor;
use crate::lib::provider::{
    command_line_provider_to_url, get_network_context, get_network_descriptor,
};
use crate::util::expiry_duration;

use anyhow::anyhow;
use clap::Clap;
use garcon::{Delay, Waiter};
use slog::warn;
use tokio::runtime::Runtime;

/// Pings an Internet Computer network and returns its status.
#[derive(Clap)]
pub struct PingOpts {
    /// The provider to use.
    /// A valid URL (starting with `http:` or `https:`) can be used here, and a special
    /// ephemeral network will be created specifically for this request. E.g.
    /// "http://localhost:12345/" is a valid network name.
    network: Option<String>,

    /// Repeatedly ping until the replica is healthy
    #[clap(long)]
    wait_healthy: bool,
}

pub fn exec(env: &dyn Environment, opts: PingOpts) -> DfxResult {
    env.get_config()
        .ok_or_else(|| anyhow!("Cannot find dfx configuration file in the current working directory. Did you forget to create one?"))?;

    // For ping, "provider" could either be a URL or a network name.
    // If not passed, we default to the "local" network.
    let network_descriptor =
        get_network_descriptor(env, opts.network).or_else::<DfxError, _>(|err| {
            let logger = env.get_logger();
            warn!(logger, "{}", err);
            let network_name = get_network_context()?;
            let url = command_line_provider_to_url(&network_name)?;
            let network_descriptor = NetworkDescriptor {
                name: "-ping-".to_string(),
                providers: vec![url],
                r#type: NetworkType::Ephemeral,
                is_ic: network_name == "ic" || network_name == DEFAULT_IC_GATEWAY,
            };
            Ok(network_descriptor)
        })?;

    let timeout = expiry_duration();
    let env = AgentEnvironment::new(env, network_descriptor, timeout)?;

    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot find dfx configuration file in the current working directory. Did you forget to create one?"))?;

    let runtime = Runtime::new().expect("Unable to create a runtime");
    if opts.wait_healthy {
        let mut waiter = Delay::builder()
            .timeout(std::time::Duration::from_secs(60))
            .throttle(std::time::Duration::from_secs(1))
            .build();
        waiter.start();

        loop {
            let status = runtime.block_on(agent.status());
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
            waiter
                .wait()
                .map_err(|_| anyhow!("Timed out waiting for replica to become healthy"))?;
        }
    } else {
        let status = runtime.block_on(agent.status())?;
        println!("{}", status);
    }

    Ok(())
}
