use anyhow::bail;
use dfx_core::network::provider::{create_network_descriptor, LocalBindDetermination};

use crate::lib::{environment::Environment, error::DfxResult};

pub(crate) fn get_pocketic_config_port(env: &dyn Environment) -> DfxResult<String> {
    let network_descriptor = create_network_descriptor(
        env.get_config()?,
        env.get_networks_config(),
        None,
        None,
        LocalBindDetermination::AsConfigured,
    )?;
    let local = network_descriptor.local_server_descriptor()?;
    match local.is_pocketic()? {
        Some(true) => {}
        Some(false) => bail!("The running server is a native replica, not PocketIC"),
        None => bail!("No PocketIC port found"),
    }
    let logger = None;
    if let Some(port) = local.get_running_pocketic_port(logger)? {
        Ok(format!("{}", port))
    } else {
        bail!("No PocketIC port found");
    }
}
