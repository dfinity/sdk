use crate::lib::provider::{create_network_descriptor, LocalBindDetermination};
use crate::{DfxResult, Environment};
use clap::Parser;

#[derive(clap::ValueEnum, Clone, Debug)]
enum InfoType {
    WebserverPort,
}

#[derive(Parser)]
#[clap(name("info"))]
pub struct InfoOpts {
    #[clap(value_enum)]
    info_type: InfoType,
}

pub fn exec(env: &dyn Environment, opts: InfoOpts) -> DfxResult {
    let value = match opts.info_type {
        InfoType::WebserverPort => get_webserver_port(env)?,
    };
    print!("{}", value);
    Ok(())
}

fn get_webserver_port(env: &dyn Environment) -> DfxResult<String> {
    let port = create_network_descriptor(
        env.get_config(),
        env.get_networks_config(),
        None, /* opts.network */
        None,
        LocalBindDetermination::ApplyRunningWebserverPort,
    )?
    .local_server_descriptor()?
    .bind_address
    .port();
    Ok(format!("{}", port))
}
