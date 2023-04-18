use crate::lib::environment::AgentEnvironment;
use crate::lib::error::DfxResult;
use crate::lib::metadata::names::{DFX_DEPS, DFX_WASM_HASH, DFX_WASM_URL};
use crate::lib::operations::canister::get_canister_status;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::{agent::create_agent_environment, environment::Environment};
use crate::NetworkOpt;
use dfx_core::config::cache::get_cache_root;
use dfx_core::config::model::dfinity::CanisterTypeProperties;
use dfx_core::identity::CallSender;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::io::Write;

use anyhow::{anyhow, bail, Context};
use candid::Principal;
use clap::Parser;
use fn_error_context::context;
use ic_agent::{Agent, AgentError};
use sha2::{Digest, Sha256};
use slog::{error, info, warn, Logger};
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

        let mut any_download_fail = false;

        for canister_id in pulled_canisters {
            if let Err(e) = download_canister_wasm(&agent_env, logger, canister_id).await {
                error!(
                    logger,
                    "Failed to download wasm of canister {canister_id}.\n{e}"
                );
                any_download_fail = true;
            }
        }

        match any_download_fail {
            true => Err(anyhow!("Some wasm download(s) failed.")),
            false => Ok(()),
        }
    })
}

#[context("Failed to fetch and parse `dfx:deps` metadata from canister {canister_id}.")]
async fn fetch_deps_to_pull(
    agent: &Agent,
    logger: &Logger,
    canister_id: Principal,
    canisters_to_pull: &mut VecDeque<Principal>,
) -> DfxResult {
    info!(logger, "Pulling canister {canister_id}...");

    match fetch_metatdata(agent, canister_id, DFX_DEPS).await {
        Ok(Some(deps_raw)) => {
            let deps = String::from_utf8(deps_raw)?;
            for entry in deps.split_terminator(';') {
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
        }
        Ok(None) => {
            warn!(
                logger,
                "`{DFX_DEPS}` metadata not found in canister {canister_id}."
            );
        }
        Err(e) => {
            bail!(e);
        }
    }
    Ok(())
}

async fn download_canister_wasm(
    agent_env: &AgentEnvironment<'_>,
    logger: &Logger,
    canister_id: Principal,
) -> DfxResult {
    info!(logger, "Downloading wasm of canister {canister_id}...");

    let agent = agent_env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    // try fetch `dfx:wasm_hash`. If not available, get the hash of the on chain canister.
    let hash_on_chain = match fetch_metatdata(agent, canister_id, DFX_WASM_HASH).await {
        Ok(Some(wasm_hash_raw)) => {
            let wasm_hash_str = String::from_utf8(wasm_hash_raw)?;
            info!(
                logger,
                "Canister {canister_id} specified a custom hash: {wasm_hash_str}"
            );
            hex::decode(wasm_hash_str)?
        }
        Ok(None) => {
            let canister_status =
                get_canister_status(agent_env, canister_id, &CallSender::SelectedId).await?;
            match canister_status.module_hash {
                Some(hash_on_chain) => hash_on_chain,
                None => {
                    bail!("Canister {canister_id} doesn't have module hash. Perhaps it's not installed.");
                }
            }
        }
        Err(e) => {
            bail!(e);
        }
    };

    // target $HOME/.cache/dfinity/wasms/{canister_id}/canister.wasm
    let wasm_dir = get_cache_root()?
        .join("wasms")
        .join(canister_id.to_string());
    let wasm_path = wasm_dir.join("canister.wasm");

    // skip download if cache hit
    if wasm_path.exists() {
        let bytes = std::fs::read(&wasm_path)?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let hash_cache = hasher.finalize();

        if hash_cache.as_slice() == hash_on_chain {
            info!(logger, "The canister wasm was found in the cache.");
            return Ok(());
        }
    }

    // fetch `dfx:wasm_url`
    let wasm_url_raw = fetch_metatdata(agent, canister_id, DFX_WASM_URL)
        .await?
        .ok_or_else(|| anyhow!("`{DFX_WASM_URL}` metadata not found in canister {canister_id}."))?;
    let wasm_url_str = String::from_utf8(wasm_url_raw)?;
    let wasm_url = reqwest::Url::parse(&wasm_url_str)?;

    // download
    let response = reqwest::get(wasm_url.clone()).await?;
    let status = response.status();
    if status.is_client_error() || status.is_server_error() {
        bail!("Failed to download wasm from url: {wasm_url}.\n  StatusCode: {status}");
    }
    let content = response.bytes().await?;

    // hash check
    let mut hasher = Sha256::new();
    hasher.update(&content);
    let hash_download = hasher.finalize();
    if hash_download.as_slice() != hash_on_chain {
        bail!(
            "Hash mismatch.
  on chain: {}
  download: {}",
            hex::encode(hash_on_chain),
            hex::encode(hash_download.as_slice())
        );
    }

    // write to a tempfile then rename
    std::fs::create_dir_all(&wasm_dir)
        .with_context(|| format!("Failed to create dir at {:?}", &wasm_dir))?;
    let mut f = tempfile::NamedTempFile::new_in(&wasm_dir)?;
    f.write_all(&content)?;
    std::fs::rename(f.path(), &wasm_path)
        .with_context(|| format!("Failed to move tempfile to {:?}", &wasm_path))?;

    Ok(())
}

#[context("Failed to fetch metadata {metadata} of canister {canister_id}.")]
async fn fetch_metatdata(
    agent: &Agent,
    canister_id: Principal,
    metadata: &str,
) -> DfxResult<Option<Vec<u8>>> {
    match agent
        .read_state_canister_metadata(canister_id, metadata)
        .await
    {
        Ok(data) => Ok(Some(data)),
        Err(agent_error) => match agent_error {
            AgentError::HttpError(ref e) => {
                let content = String::from_utf8(e.content.clone())?;
                if content.starts_with("Custom section") {
                    Ok(None)
                } else {
                    bail!(agent_error);
                }
            }
            _ => bail!(agent_error),
        },
    }
}
