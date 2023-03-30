use std::path::Path;

use crate::{
    config::model::{
        local_server_descriptor::LocalServerDescriptor, network_descriptor::NetworkDescriptor,
    },
    error::{network_config::NetworkConfigError, uri::UriError},
};

use slog::{info, Logger};
use url::Url;

/// Gets a list of replica URLs
pub fn get_replica_urls(
    logger: &Logger,
    network_descriptor: &NetworkDescriptor,
) -> Result<Vec<Url>, NetworkConfigError> {
    if network_descriptor.name == "local" {
        let local_server_descriptor = network_descriptor.local_server_descriptor()?;

        if let Some(port) = get_running_replica_port(Some(logger), local_server_descriptor)? {
            let mut socket_addr = local_server_descriptor.bind_address;
            socket_addr.set_port(port);
            let url = format!("http://{}", socket_addr);
            let url = Url::parse(&url).map_err(|e| UriError::UrlParseError(url.to_string(), e))?;
            return Ok(vec![url]);
        }
    }
    network_descriptor.replica_endpoints()
}

/// Gets the port of a local replica.
///
/// # Prerequisites
/// - A local replica or emulator needs to be running, e.g. with `dfx start`.
pub fn get_running_replica_port(
    logger: Option<&Logger>,
    local_server_descriptor: &LocalServerDescriptor,
) -> Result<Option<u16>, UriError> {
    // dfx start and dfx replica both write these as empty, and then
    // populate one with a port.
    let emulator_port_path = local_server_descriptor.ic_ref_port_path();
    let replica_port_path = local_server_descriptor.replica_port_path();

    match read_port_from(&replica_port_path)? {
        Some(port) => {
            if let Some(logger) = logger {
                info!(logger, "Found local replica running on port {}", port);
            }
            Ok(Some(port))
        }
        None => match read_port_from(&emulator_port_path)? {
            Some(port) => {
                if let Some(logger) = logger {
                    info!(logger, "Found local emulator running on port {}", port);
                }
                Ok(Some(port))
            }
            None => Ok(None),
        },
    }
}

/// Reads a port number from a file.
///
/// # Prerequisites
/// The file is expected to contain the port number only, as utf8 text.
fn read_port_from(path: &Path) -> Result<Option<u16>, UriError> {
    if path.exists() {
        let s = crate::fs::read_to_string(path)?;
        let s = s.trim();
        if s.is_empty() {
            Ok(None)
        } else {
            let port = s
                .parse::<u16>()
                .map_err(|e| UriError::PortReadError(path.to_path_buf(), e))?;
            Ok(Some(port))
        }
    } else {
        Ok(None)
    }
}
