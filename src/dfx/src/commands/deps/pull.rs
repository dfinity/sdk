use crate::lib::deps::{
    copy_service_candid_to_project, get_pulled_wasm_path, get_service_candid_path, save_pulled_json,
};
use crate::lib::deps::{PulledCanister, PulledJson};
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_utils::CallSender;
use crate::lib::metadata::names::{
    CANDID_SERVICE, DFX_DEPS, DFX_INIT, DFX_WASM_HASH, DFX_WASM_URL,
};
use crate::lib::operations::canister::get_canister_status;
use crate::lib::root_key::fetch_root_key_if_needed;
use dfx_core::config::cache::get_cache_root;
use dfx_core::fs::composite::ensure_dir_exists;
use std::collections::VecDeque;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{anyhow, bail, Context};
use candid::Principal;
use clap::Parser;
use fn_error_context::context;
use ic_agent::{Agent, AgentError};
use sha2::{Digest, Sha256};
use slog::{error, info, warn, Logger};

/// Pull canisters upon which the project depends
#[derive(Parser)]
pub struct DepsPullOpts {}

pub async fn exec(env: &dyn Environment, _opts: DepsPullOpts) -> DfxResult {
    let logger = env.get_logger();

    fetch_root_key_if_needed(env).await?;

    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    let pull_canisters_in_config = env
        .get_config_or_anyhow()?
        .get_config()
        .get_pull_canisters()?;

    let mut canisters_to_resolve: VecDeque<Principal> =
        pull_canisters_in_config.values().cloned().collect();

    let mut pulled_json = PulledJson::with_named(&pull_canisters_in_config);

    while let Some(callee_canister) = canisters_to_resolve.pop_front() {
        if !pulled_json.canisters.contains_key(&callee_canister) {
            fetch_deps_to_pull(
                agent,
                logger,
                callee_canister,
                &mut canisters_to_resolve,
                &mut pulled_json,
            )
            .await?;
        }
    }

    let mut message = String::new();
    message.push_str(&format!(
        "Found {} dependencies:",
        pulled_json.canisters.len()
    ));
    for id in pulled_json.canisters.keys() {
        message.push('\n');
        message.push_str(&id.to_text());
    }

    info!(logger, "{}", message);

    let mut any_download_fail = false;

    for (canister_id, pulled_canister) in pulled_json.canisters.iter_mut() {
        if let Err(e) = download_canister_files(env, logger, *canister_id, pulled_canister).await {
            error!(logger, "Failed to pull canister {canister_id}.\n{e}");
            any_download_fail = true;
        }
    }

    match any_download_fail {
        true => bail!("Failed when pulling canisters."),
        false => {
            for (name, canister_id) in &pull_canisters_in_config {
                copy_service_candid_to_project(env, name, *canister_id)?;
            }
        }
    }

    save_pulled_json(env, &pulled_json)?;
    Ok(())
}

#[context("Failed to fetch and parse `dfx:deps` metadata from canister {canister_id}.")]
async fn fetch_deps_to_pull(
    agent: &Agent,
    logger: &Logger,
    canister_id: Principal,
    canisters_to_pull: &mut VecDeque<Principal>,
    pulled_json: &mut PulledJson,
) -> DfxResult {
    info!(
        logger,
        "Resolving dependencies of canister {canister_id}..."
    );

    match fetch_metatdata(agent, canister_id, DFX_DEPS).await {
        Ok(Some(deps_raw)) => {
            let deps_str = String::from_utf8(deps_raw)?;
            let mut deps = vec![];
            for entry in deps_str.split_terminator(';') {
                match entry.split_once(':') {
                    Some((_, p)) => {
                        let dep_id = Principal::from_text(p)
                            .with_context(|| format!("`{p}` is not a valid Principal."))?;
                        canisters_to_pull.push_back(dep_id);
                        deps.push(dep_id);
                    }
                    None => bail!(
                        "Failed to parse `dfx:deps` entry: {entry}. Expected `name:Principal`. "
                    ),
                }
            }
            pulled_json.canisters.insert(
                canister_id,
                PulledCanister {
                    deps,
                    ..Default::default()
                },
            );
        }
        Ok(None) => {
            warn!(
                logger,
                "`{DFX_DEPS}` metadata not found in canister {canister_id}."
            );
            pulled_json
                .canisters
                .insert(canister_id, PulledCanister::default());
        }
        Err(e) => {
            bail!(e);
        }
    }
    Ok(())
}

// download canister wasm, candid, init
async fn download_canister_files(
    env: &dyn Environment,
    logger: &Logger,
    canister_id: Principal,
    pulled_canister: &mut PulledCanister,
) -> DfxResult {
    info!(logger, "Pulling canister {canister_id}...");

    let agent = env
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
                get_canister_status(env, canister_id, &CallSender::SelectedId).await?;
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

    pulled_canister.wasm_hash = hex::encode(&hash_on_chain);

    // will save wasm and candid in $(cache_root)/pulled/{canister_id}/
    let canister_dir = get_cache_root()?
        .join("pulled")
        .join(canister_id.to_string());
    std::fs::create_dir_all(&canister_dir)
        .with_context(|| format!("Failed to create dir at {:?}", &canister_dir))?;

    let wasm_path = get_pulled_wasm_path(canister_id)?;

    // skip download if cache hit
    let mut cache_hit = false;
    if wasm_path.exists() {
        let bytes = std::fs::read(&wasm_path)?;
        let hash_cache = Sha256::digest(bytes);

        if hash_cache.as_slice() == hash_on_chain {
            cache_hit = true;
            info!(logger, "The canister wasm was found in the cache.");
        }
    }
    if !cache_hit {
        // fetch `dfx:wasm_url`
        let wasm_url_raw = fetch_metatdata(agent, canister_id, DFX_WASM_URL)
            .await?
            .ok_or_else(|| {
                anyhow!("`{DFX_WASM_URL}` metadata not found in canister {canister_id}.")
            })?;
        let wasm_url_str = String::from_utf8(wasm_url_raw)?;
        let wasm_url = reqwest::Url::parse(&wasm_url_str)?;

        pulled_canister.wasm_url = Some(wasm_url_str);

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

        write_to_tempfile_then_rename(&content, &wasm_path)?;
    }

    // fetch `candid:service` and save it
    let service_candid_bytes = fetch_metatdata(agent, canister_id, CANDID_SERVICE)
        .await?
        .ok_or_else(|| {
            anyhow!("`{CANDID_SERVICE}` metadata not found in canister {canister_id}.")
        })?;
    let service_candid_path = get_service_candid_path(canister_id)?;
    write_to_tempfile_then_rename(&service_candid_bytes, &service_candid_path)?;

    // try fetch `dfx:init`
    match fetch_metatdata(agent, canister_id, DFX_INIT).await {
        Ok(Some(init_bytes)) => {
            // write_to_tempfile_then_rename(&init_bytes, &canister_dir, "init.txt")?
            pulled_canister.init = Some(String::from_utf8(init_bytes)?);
        }
        Ok(None) => {
            info!(
                logger,
                "Canister {canister_id} doesn't define `{DFX_INIT}` metadata."
            );
            pulled_canister.init = None;
        }
        Err(e) => {
            bail!(e);
        }
    };

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

#[context("Failed to write to a tempfile then rename it to {}", path.display())]
fn write_to_tempfile_then_rename(content: &[u8], path: &PathBuf) -> DfxResult {
    assert!(path.is_absolute());
    let dir = path
        .parent()
        .ok_or_else(|| anyhow!("Failed to get the parent dir from path"))?;
    ensure_dir_exists(&dir)?;
    let mut f = tempfile::NamedTempFile::new_in(dir)?;
    f.write_all(content)?;
    std::fs::rename(f.path(), path)?;
    Ok(())
}
