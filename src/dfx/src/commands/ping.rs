use crate::config::dfinity::NetworkType;
use crate::lib::environment::{AgentEnvironment, Environment};
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::network::network_descriptor::NetworkDescriptor;
use crate::lib::provider::{
    command_line_provider_to_url, get_network_context, get_network_descriptor,
};
use crate::util::expiry_duration;

use anyhow::anyhow;
use clap::Clap;
use slog::warn;
use tokio::runtime::Runtime;

/// Pings an Internet Computer network and returns its status.
#[derive(Clap)]
pub struct PingOpts {
    /// The provider to use.
    network: Option<String>,
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
                is_local: false,
            };
            Ok(network_descriptor)
        })?;

    let timeout = expiry_duration();
    let env = AgentEnvironment::new(env, network_descriptor, timeout)?;

    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot find dfx configuration file in the current working directory. Did you forget to create one?"))?;

    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    let status = runtime.block_on(agent.status())?;
    println!("{}", status);

    Ok(())
}
