use crate::lib::agent::create_anonymous_agent_environment;
use crate::lib::deps::pull::{
    copy_service_candid_to_project, download_all_and_generate_pulled_json, resolve_all_dependencies,
};
use crate::lib::deps::{get_pull_canisters_in_config, save_pulled_json};
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::network::network_opt::NetworkOpt;
use crate::lib::root_key::fetch_root_key_if_needed;
use anyhow::anyhow;
use clap::Parser;
use slog::info;

/// Pull canisters upon which the project depends.
/// This command connects to the "ic" mainnet by default.
/// You can still choose other network by setting `--network`.
#[derive(Parser)]
pub struct DepsPullOpts {
    #[command(flatten)]
    network: NetworkOpt,
}

pub async fn exec(env: &dyn Environment, opts: DepsPullOpts) -> DfxResult {
    let logger = env.get_logger();
    let pull_canisters_in_config = get_pull_canisters_in_config(env)?;
    if pull_canisters_in_config.is_empty() {
        info!(logger, "There are no pull dependencies defined in dfx.json");
        return Ok(());
    }

    let network = opts
        .network
        .to_network_name()
        .unwrap_or_else(|| "ic".to_string());
    let env = create_anonymous_agent_environment(env, Some(network))?;

    let project_root = env.get_config_or_anyhow()?.get_project_root().to_path_buf();

    fetch_root_key_if_needed(&env).await?;

    let agent = env.get_agent();

    let all_dependencies =
        resolve_all_dependencies(agent, logger, &pull_canisters_in_config).await?;

    let mut pulled_json =
        download_all_and_generate_pulled_json(agent, logger, &all_dependencies).await?;

    for (name, canister_id) in &pull_canisters_in_config {
        copy_service_candid_to_project(&project_root, name, canister_id)?;
        let pulled_canister = pulled_json
            .canisters
            .get_mut(canister_id)
            .ok_or_else(|| anyhow!("Failed to find {canister_id} entry in pulled.json"))?;
        pulled_canister.name = Some(name.clone());
    }

    save_pulled_json(&project_root, &pulled_json)?;
    Ok(())
}
