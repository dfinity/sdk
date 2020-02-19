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
struct ClientTomlConfig<'a> {
    http_handler: HttpHandlerConfig<'a>,
    state_root: StateManagerConfig<'a>,
}

pub fn generate_client_configuration(
    port_file_path: &PathBuf,
    state_root: &PathBuf,
) -> DfxResult<String> {
    let http_values = ClientTomlConfig {
        http_handler: HttpHandlerConfig {
            write_port_to: port_file_path,
        },
        state_root: StateManagerConfig { state_root },
    };
    toml::to_string(&http_values).map_err(DfxError::CouldNotSerializeClientConfiguration)
}
