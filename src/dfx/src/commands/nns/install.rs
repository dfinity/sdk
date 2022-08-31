use crate::lib::error::DfxResult;
use crate::Environment;
use anyhow::anyhow;

use crate::lib::nns::install_nns::install_nns;
use crate::lib::root_key::fetch_root_key_if_needed;
use clap::Parser;

/// Installs the nns canisters
#[derive(Parser)]
pub struct InstallOpts {}

pub async fn exec(env: &dyn Environment, _opts: InstallOpts) -> DfxResult {
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    fetch_root_key_if_needed(env).await?;

    let network_descriptor = env.get_network_descriptor();
    let local_server_descriptor = network_descriptor.local_server_descriptor()?;
    let replicated_state_dir = local_server_descriptor.replicated_state_dir();
    let provider_url = network_descriptor.first_provider()?;

    let ic_nns_init_path = env.get_cache().get_binary_command_path("ic-nns-init")?;

    install_nns(
        agent,
        provider_url,
        &ic_nns_init_path,
        &replicated_state_dir,
    )
    .await
}
