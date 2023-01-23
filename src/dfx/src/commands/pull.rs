use crate::lib::error::DfxResult;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::{environment::Environment, provider::create_agent_environment};
use crate::NetworkOpt;
use dfx_core::config::model::dfinity::CanisterTypeProperties;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::{anyhow, bail, Context};
use candid::Principal;
use clap::Parser;
use fn_error_context::context;
use ic_agent::{Agent, AgentError};
use slog::Logger;
use tokio::runtime::Runtime;

/// Pull canisters upon which the project depends
#[derive(Parser)]
pub struct PullOpts {
    /// Specifies the name of the canister you want to pull.
    /// If you don’t specify a canister name, all pull type canisters defined in the dfx.json file are pulled.
    canister_name: Option<String>,

    #[clap(flatten)]
    network: NetworkOpt,
}

pub fn exec(env: &dyn Environment, opts: PullOpts) -> DfxResult {
    let agent_env = create_agent_environment(env, opts.network.network)?;
    let logger = env.get_logger();

    let runtime = Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(async {
        fetch_root_key_if_needed(&agent_env).await?;

        let agent = agent_env
            .get_agent()
            .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

        let config = agent_env.get_config_or_anyhow()?;
        let mut pull_canisters = BTreeMap::new();

        if let Some(map) = &config.get_config().canisters {
            for (k, v) in map {
                if let CanisterTypeProperties::Pull { id } = v.type_specific {
                    pull_canisters.insert(k, id);
                }
            }
        };

        let mut canisters_to_pull: VecDeque<Principal> = match opts.canister_name {
            Some(s) => match pull_canisters.get(&s) {
                Some(v) => VecDeque::from([*v]),
                None => bail!("There is no pull type canister \"{s}\" defined in dfx.json"),
            },
            None => pull_canisters.values().cloned().collect(),
        };

        let mut pulled_canisters: BTreeSet<Principal> = BTreeSet::new();

        while let Some(callee_canister) = canisters_to_pull.pop_front() {
            if !pulled_canisters.contains(&callee_canister) {
                pulled_canisters.insert(callee_canister);
                fetch_deps_to_pull(agent, logger, callee_canister, &mut canisters_to_pull).await?;
            }
        }

        Ok(())
    })
}

#[context("Failed while fetch and parse `dfx:deps` metadata from canister {canister_id}.")]
async fn fetch_deps_to_pull(
    agent: &Agent,
    logger: &Logger,
    canister_id: Principal,
    canisters_to_pull: &mut VecDeque<Principal>,
) -> DfxResult {
    slog::info!(logger, "Pulling canister {canister_id}...");

    match agent
        .read_state_canister_metadata(canister_id, "dfx:deps")
        .await
    {
        Ok(data) => {
            let data = String::from_utf8(data)?;
            for entry in data.split_terminator(';') {
                match entry.split_once(':') {
                    Some((_, p)) => {
                        let canister_id = Principal::from_text(p)
                            .with_context(|| format!("`{p}` is not a valid Principal."))?;
                        canisters_to_pull.push_back(canister_id);
                    }
                    None => bail!(
                        "Failed to parse `dfx:deps` entry: {entry}. Expected `name:Principal`. "
                    ),
                }
            }
            Ok(())
        }
        Err(agent_error) => match agent_error {
            AgentError::HttpError(ref e) => {
                let content = String::from_utf8(e.content.clone())?;
                if content.starts_with("Custom section") {
                    slog::warn!(
                        logger,
                        "`dfx:deps` metadata not found in canister {canister_id}."
                    );
                    Ok(())
                } else {
                    Err(anyhow!(agent_error))
                }
            }
            _ => Err(anyhow!(agent_error)),
        },
    }
}
