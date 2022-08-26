use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::network::local_server_descriptor::LocalServerDescriptor;
use crate::lib::network::network_descriptor::NetworkDescriptor;

use fn_error_context::context;
use slog::info;
use std::path::Path;
use url::Url;

/// Gets the port of a local replica.
///
/// # Prerequisites
/// - A local replica or emulator needs to be running, e.g. with `dfx start`.
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

/// Reads a port number from a file.
///
/// # Prerequisites
/// The file is expected to contain the port number only, as utf8 text.
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

/// Gets the list of compute provider API endpoints.
#[context("Failed to get providers for network '{}'.", network_descriptor.name)]
pub fn get_providers(network_descriptor: &NetworkDescriptor) -> DfxResult<Vec<Url>> {
    network_descriptor
        .providers
        .iter()
        .map(|url| parse_url(url))
        .collect()
}

/// Gets a list of replica URLs
#[context("Failed to determine replica urls.")]
pub fn get_replica_urls(
    env: &dyn Environment,
    network_descriptor: &NetworkDescriptor,
) -> DfxResult<Vec<Url>> {
    if network_descriptor.name == "local" {
        let local_server_descriptor = network_descriptor.local_server_descriptor()?;
        if let Some(port) = get_running_replica_port(env, local_server_descriptor)? {
            let mut socket_addr = local_server_descriptor.bind_address;
            socket_addr.set_port(port);
            let url = format!("http://{}", socket_addr);
            let url = Url::parse(&url)?;
            return Ok(vec![url]);
        }
    }
    get_providers(network_descriptor)
}

/// Parses a URL, returning a DfxResult instead of a Url::ParseError.
#[context("Failed to parse url '{}'.", url)]
fn parse_url(url: &str) -> DfxResult<Url> {
    Ok(Url::parse(url)?)
}
