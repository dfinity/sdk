use crate::lib::error::DfxResult;
use crate::Environment;
use anyhow::bail;
use dfx_core::network::provider::{create_network_descriptor, LocalBindDetermination};

pub(crate) fn get_replica_port(env: &dyn Environment) -> DfxResult<String> {
    let network_descriptor = create_network_descriptor(
        env.get_config()?,
        env.get_networks_config(),
        None,
        None,
        LocalBindDetermination::AsConfigured,
    )?;
    let local = network_descriptor.local_server_descriptor()?;
    match local.is_pocketic()? {
        Some(false) => {}
        Some(true) => bail!("The running server is PocketIC, not a native replica"),
        None => bail!("No replica port found"),
    }
    let logger = None;
    if let Some(port) = local.get_running_replica_port(logger)? {
        Ok(format!("{}", port))
    } else {
        bail!("No replica port found");
    }
}
