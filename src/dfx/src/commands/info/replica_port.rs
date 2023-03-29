use crate::lib::error::DfxResult;
use crate::Environment;
use dfx_core::network::provider::{create_network_descriptor, LocalBindDetermination};
use dfx_core::network::uri::get_running_replica_port;

use anyhow::bail;

pub(crate) fn get_replica_port(env: &dyn Environment) -> DfxResult<String> {
    let network_descriptor = create_network_descriptor(
        env.get_config(),
        env.get_networks_config(),
        None,
        None,
        LocalBindDetermination::AsConfigured,
    )?;

    if let Some(port) =
        get_running_replica_port(None, network_descriptor.local_server_descriptor()?)?
    {
        Ok(format!("{}", port))
    } else {
        bail!("No replica port found");
    }
}
