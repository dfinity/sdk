use crate::lib::error::DfxResult;
use crate::Environment;
use dfx_core::network::provider::{create_network_descriptor, LocalBindDetermination};

pub(crate) fn get_webserver_port(env: &dyn Environment) -> DfxResult<String> {
    let port = create_network_descriptor(
        env.get_config()?,
        env.get_networks_config(),
        None,
        None,
        LocalBindDetermination::ApplyRunningWebserverPort,
    )?
    .local_server_descriptor()?
    .bind_address
    .port();
    Ok(format!("{}", port))
}
