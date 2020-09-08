use crate::config::dfinity::NetworkType;
use crate::lib::environment::{AgentEnvironment, Environment};
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::lib::network::network_descriptor::NetworkDescriptor;
use crate::lib::provider::{command_line_provider_to_url, get_network_descriptor};
use clap::{App, Arg, ArgMatches, SubCommand};
use tokio::runtime::Runtime;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("ping")
        .about(UserMessage::Ping.to_str())
        .arg(
            Arg::with_name("network")
                .help("The provider to use.")
                .takes_value(true),
        )
}

pub fn exec(env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult {
    env.get_config()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;

    // For ping, "provider" could either be a URL or a network name.
    // If not passed, we default to the "local" network.
    let network_descriptor =
        get_network_descriptor(env, args).or_else::<DfxError, _>(|err| match err {
            DfxError::ComputeNetworkNotFound(network_name) => {
                let url = command_line_provider_to_url(&network_name)?;
                let network_descriptor = NetworkDescriptor {
                    name: "-ping-".to_string(),
                    providers: vec![url],
                    r#type: NetworkType::Ephemeral,
                };
                Ok(network_descriptor)
            }
            other => Err(other),
        })?;

    let env = AgentEnvironment::new(env, network_descriptor)?;

    let agent = env
        .get_agent()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;

    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    let status = runtime.block_on(agent.status())?;
    println!("{}", status);

    Ok(())
}
