use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::network::local_server_descriptor::LocalServerDescriptor;

use fn_error_context::context;
use slog::info;
use std::path::Path;

pub fn get_running_replica_port(
    env: &dyn Environment,
    local_server_descriptor: &LocalServerDescriptor,
) -> DfxResult<Option<u16>> {
    let logger = env.get_logger();
    // dfx start and dfx replica both write these as empty, and then
    // populate one with a port.
    let emulator_port_path = local_server_descriptor.ic_ref_port_path();
    let replica_port_path = local_server_descriptor.replica_port_path();

    match read_port_from(&replica_port_path)? {
        Some(port) => {
            info!(logger, "Found local replica running on port {}", port);
            Ok(Some(port))
        }
        None => match read_port_from(&emulator_port_path)? {
            Some(port) => {
                info!(logger, "Found local emulator running on port {}", port);
                Ok(Some(port))
            }
            None => Ok(None),
        },
    }
}

#[context("Failed to read port value from {}", path.to_string_lossy())]
fn read_port_from(path: &Path) -> DfxResult<Option<u16>> {
    if path.exists() {
        let s = std::fs::read_to_string(&path)?;
        let s = s.trim();
        if s.is_empty() {
            Ok(None)
        } else {
            let port = s.parse::<u16>()?;
            Ok(Some(port))
        }
    } else {
        Ok(None)
    }
}
