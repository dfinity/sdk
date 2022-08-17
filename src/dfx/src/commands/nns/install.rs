use crate::{DfxResult, Environment};
use anyhow::anyhow;

use crate::lib::operations::nns::install_nns::install_nns;
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

    let ic_nns_init_path = env.get_cache().get_binary_command_path("ic-nns-init")?;
    let replicated_state_dir = env
        .get_network_descriptor()
        .local_server_descriptor()?
        .replicated_state_dir();

    install_nns(agent, &ic_nns_init_path, &replicated_state_dir).await
}
