use crate::lib::error::DfxResult;
use crate::lib::network::local_server_descriptor::LocalNetworkScopeDescriptor;
use crate::lib::network::network_descriptor::NetworkDescriptor;

use anyhow::Context;
use fn_error_context::context;
use std::path::Path;

/// A cohesive network directory is one in which the directory in question contains
/// a file `network-id`, which contains the same contents as the `network-id` file
/// in the network data directory.  In this way, after `dfx start --clean`, we
/// can later clean up data in project directories.
#[context("Failed to ensure cohesive network directory at {}", directory.display())]
pub fn ensure_cohesive_network_directory(
    network_descriptor: &NetworkDescriptor,
    directory: &Path,
) -> DfxResult {
    let scope = network_descriptor
        .local_server_descriptor
        .as_ref()
        .map(|d| &d.scope);

    if let Some(LocalNetworkScopeDescriptor::Shared { network_id_path }) = &scope {
        if network_id_path.is_file() {
            let network_id = std::fs::read_to_string(network_id_path)
                .with_context(|| format!("unable to read {}", network_id_path.display()))?;
            let project_network_id_path = directory.join("network-id");
            let reset = directory.is_dir()
                && (!project_network_id_path.exists()
                    || std::fs::read_to_string(&project_network_id_path)? != network_id);

            if reset {
                std::fs::remove_dir_all(&directory).with_context(|| {
                    format!("Cannot remove directory at '{}'", directory.display())
                })?;
            };

            if !directory.exists() {
                std::fs::create_dir_all(&directory).with_context(|| {
                    format!(
                        "Failed to create directory {}.",
                        directory.to_string_lossy()
                    )
                })?;
                std::fs::write(&project_network_id_path, &network_id)?;
            }
        }
    }

    Ok(())
}
