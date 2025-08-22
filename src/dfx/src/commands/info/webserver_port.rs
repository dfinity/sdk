use crate::Environment;
use crate::lib::error::DfxResult;
use dfx_core::network::provider::{LocalBindDetermination, create_network_descriptor};

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
