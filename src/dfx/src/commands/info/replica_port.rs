use crate::lib::error::DfxResult;
use crate::Environment;
use anyhow::bail;
use dfx_core::{
    config::model::replica_config::CachedReplicaConfig,
    network::provider::{create_network_descriptor, LocalBindDetermination},
};

pub(crate) fn get_replica_port(env: &dyn Environment) -> DfxResult<String> {
    let network_descriptor = create_network_descriptor(
        env.get_config()?,
        env.get_networks_config(),
        None,
        None,
        LocalBindDetermination::AsConfigured,
    )?;
    if let Some(l) = &network_descriptor.local_server_descriptor {
        if l.effective_config()?
            .is_some_and(|c| matches!(c.config, CachedReplicaConfig::PocketIc { .. }))
        {
            return super::get_webserver_port(env);
        }
    }

    let logger = None;
    if let Some(port) = network_descriptor
        .local_server_descriptor()?
        .get_running_replica_port(logger)?
    {
        Ok(format!("{}", port))
    } else {
        bail!("No replica port found");
    }
}
