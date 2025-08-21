use crate::Environment;
use crate::lib::environment::AgentEnvironment;
use crate::lib::error::DfxResult;
use dfx_core::identity::ANONYMOUS_IDENTITY_NAME;
use dfx_core::network::provider::{LocalBindDetermination, create_network_descriptor};

use dfx_core::util::expiry_duration;
use fn_error_context::context;

#[context("Failed to create AgentEnvironment.")]
pub fn create_agent_environment<'a>(
    env: &'a (dyn Environment + 'a),
    network: Option<String>,
) -> DfxResult<AgentEnvironment<'a>> {
    let network_descriptor = create_network_descriptor(
        env.get_config()?,
        env.get_networks_config(),
        network,
        None,
        LocalBindDetermination::ApplyRunningWebserverPort,
    )?;
    let timeout = expiry_duration();
    AgentEnvironment::new(env, network_descriptor, timeout, None)
}

pub fn create_anonymous_agent_environment<'a>(
    env: &'a (dyn Environment + 'a),
    network: Option<String>,
) -> DfxResult<AgentEnvironment<'a>> {
    let network_descriptor = create_network_descriptor(
        env.get_config()?,
        env.get_networks_config(),
        network,
        None,
        LocalBindDetermination::ApplyRunningWebserverPort,
    )?;
    let timeout = expiry_duration();
    AgentEnvironment::new(
        env,
        network_descriptor,
        timeout,
        Some(ANONYMOUS_IDENTITY_NAME),
    )
}
