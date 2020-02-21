use crate::lib::error::{DfxError, DfxResult};

use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
struct HttpHandlerConfig<'a> {
    write_port_to: &'a PathBuf,
}
#[derive(Debug, Serialize)]
struct StateManagerConfig<'a> {
    state_root: &'a PathBuf,
}

#[derive(Debug, Serialize)]
pub struct ClientTomlConfig<'a> {
    state_manager: StateManagerConfig<'a>,
    http_handler: HttpHandlerConfig<'a>,
}

pub fn generate_client_configuration(
    port_file_path: &PathBuf,
    state_root: &PathBuf,
) -> DfxResult<String> {
    let config = ClientTomlConfig {
        http_handler: HttpHandlerConfig {
            write_port_to: port_file_path,
        },
        state_manager: StateManagerConfig { state_root },
    };
    toml::to_string(&config).map_err(DfxError::CouldNotSerializeClientConfiguration)
}
