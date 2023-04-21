use crate::lib::deps::{
    get_candid_path_in_project, get_pull_canisters_in_config, get_pulled_service_candid_path,
    get_pulled_wasm_path, get_pulled_wasm_url_txt_path, save_pulled_json,
};
use crate::lib::deps::{PulledCanister, PulledJson};
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::metadata::names::{
    CANDID_ARGS, CANDID_SERVICE, DFX_DEPS, DFX_INIT, DFX_WASM_HASH, DFX_WASM_URL,
};
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::state_tree::canister_info::read_state_tree_canister_module_hash;
use dfx_core::config::cache::get_cache_root;
use dfx_core::fs::composite::{ensure_dir_exists, ensure_parent_dir_exists};

use std::collections::VecDeque;
use std::io::Write;
use std::path::Path;

use anyhow::{anyhow, bail, Context};
use candid::Principal;
use clap::Parser;
use fn_error_context::context;
use ic_agent::{Agent, AgentError};
// TODO: update the usage of this method once ic-wasm bump (#3090)
use ic_wasm::metadata::get_metadata;
use sha2::{Digest, Sha256};
use slog::{error, info, trace, warn, Logger};

/// Pull canisters upon which the project depends
#[derive(Parser)]
pub struct DepsPullOpts {}

pub async fn exec(env: &dyn Environment, _opts: DepsPullOpts) -> DfxResult {
    let logger = env.get_logger();
    let pull_canisters_in_config = get_pull_canisters_in_config(env)?;
    if pull_canisters_in_config.is_empty() {
        info!(logger, "There are no pull dependencies defined in dfx.json");
        return Ok(());
    }

    let project_root = env.get_config_or_anyhow()?.get_project_root().to_path_buf();

    fetch_root_key_if_needed(env).await?;

    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

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
        pulled_json.num_of_canisters()
    ));
    for id in pulled_json.get_all_ids() {
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

    if any_download_fail {
        bail!("Failed when pulling canisters.");
    }

    for (name, canister_id) in &pull_canisters_in_config {
        copy_service_candid_to_project(&project_root, name, *canister_id)?;
    }

    save_pulled_json(&project_root, &pulled_json)?;
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

    match fetch_metadata(agent, canister_id, DFX_DEPS).await? {
        Some(deps_raw) => {
            let deps_str = String::from_utf8(deps_raw)?;
            let deps = parse_dfx_deps(&deps_str)?;
            canisters_to_pull.extend(deps.iter().copied());
            pulled_json.canisters.insert(
                canister_id,
                PulledCanister {
                    deps,
                    ..Default::default()
                },
            );
        }
        None => {
            warn!(
                logger,
                "`{DFX_DEPS}` metadata not found in canister {canister_id}."
            );
            pulled_json
                .canisters
                .insert(canister_id, PulledCanister::default());
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
    let hash_on_chain = match fetch_metadata(agent, canister_id, DFX_WASM_HASH).await? {
        Some(wasm_hash_raw) => {
            let wasm_hash_str = String::from_utf8(wasm_hash_raw)?;
            trace!(
                logger,
                "Canister {canister_id} specified a custom hash: {wasm_hash_str}"
            );
            hex::decode(wasm_hash_str)?
        }
        None => {
            match read_state_tree_canister_module_hash(agent, canister_id).await? {
                Some(hash_on_chain) => hash_on_chain,
                None => {
                    bail!("Canister {canister_id} doesn't have module hash. Perhaps it's not installed.");
                }
            }
        }
    };

    pulled_canister.wasm_hash = hex::encode(&hash_on_chain);

    // will save wasm and candid in $(cache_root)/pulled/{canister_id}/
    let canister_dir = get_cache_root()?
        .join("pulled")
        .join(canister_id.to_string());
    dfx_core::fs::create_dir_all(&canister_dir)?;

    let wasm_path = get_pulled_wasm_path(canister_id)?;

    // skip download if cache hit
    let mut cache_hit = false;
    let wasm_url_txt_path = get_pulled_wasm_url_txt_path(canister_id)?;
    if wasm_path.exists() {
        let bytes = dfx_core::fs::read(&wasm_path)?;
        let hash_cache = Sha256::digest(bytes);
        if hash_cache.as_slice() == hash_on_chain {
            cache_hit = true;
            trace!(logger, "The canister wasm was found in the cache.");
            let wasm_url_str = dfx_core::fs::read_to_string(&wasm_url_txt_path)
                .with_context(|| format!("Failed to read {:?}", &wasm_url_txt_path))?;
            pulled_canister.wasm_url = Some(wasm_url_str);
        }
    }
    if !cache_hit {
        // fetch `dfx:wasm_url`
        let wasm_url_raw = fetch_metadata(agent, canister_id, DFX_WASM_URL)
            .await?
            .ok_or_else(|| {
                anyhow!("`{DFX_WASM_URL}` metadata not found in canister {canister_id}.")
            })?;
        let wasm_url_str = String::from_utf8(wasm_url_raw)?;
        let wasm_url = reqwest::Url::parse(&wasm_url_str)?;
        write_to_tempfile_then_rename(wasm_url_str.as_bytes(), &wasm_url_txt_path)?;
        pulled_canister.wasm_url = Some(wasm_url_str);

        // download
        let response = reqwest::get(wasm_url.clone()).await?;
        let status = response.status();
        if status.is_client_error() || status.is_server_error() {
            bail!("Failed to download wasm from url: {wasm_url}.\n  StatusCode: {status}");
        }
        let content = response.bytes().await?;

        // hash check
        let hash_download = Sha256::digest(&content);
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
    // get `candid:service` from downloaded wasm
    let wasm = dfx_core::fs::read(&wasm_path).context("Failed to read wasm")?;
    let candid_service = get_metadata(&wasm, CANDID_SERVICE).with_context(|| {
        format!(
            "Failed to get {} metadata from {:?}",
            CANDID_SERVICE, &wasm_path
        )
    })?;
    let service_candid_path = get_pulled_service_candid_path(canister_id)?;
    write_to_tempfile_then_rename(candid_service.as_bytes(), &service_candid_path)?;

    // get `candid:args` from downloaded wasm
    let candid_args = get_metadata(&wasm, CANDID_ARGS).with_context(|| {
        format!(
            "Failed to get {} metadata from {:?}",
            CANDID_ARGS, &wasm_path
        )
    })?;
    pulled_canister.candid_args = Some(candid_args);

    // try get `dfx:init` from downloaded wasm
    match get_metadata(&wasm, DFX_INIT) {
        Ok(dfx_init) => pulled_canister.dfx_init = Some(dfx_init),
        Err(_e) => trace!(
            logger,
            "Failed to get {} metadata from {:?}",
            DFX_INIT,
            &wasm_path
        ),
    }

    Ok(())
}

#[context("Failed to fetch metadata {metadata} of canister {canister_id}.")]
async fn fetch_metadata(
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
            // replica returns such error
            AgentError::HttpError(ref e) => {
                let status = e.status;
                let content = String::from_utf8(e.content.clone())?;
                if status == 404
                    && content.starts_with(&format!("Custom section {metadata} not found"))
                {
                    Ok(None)
                } else {
                    bail!(agent_error);
                }
            }
            // ic-ref returns such error when the canister doesn't define the metadata
            AgentError::LookupPathAbsent(_) => Ok(None),
            _ => {
                bail!(agent_error)
            }
        },
    }
}

#[context("Failed to write to a tempfile then rename it to {}", path.display())]
fn write_to_tempfile_then_rename(content: &[u8], path: &Path) -> DfxResult {
    assert!(path.is_absolute());
    let dir = dfx_core::fs::parent(path)?;
    ensure_dir_exists(&dir)?;
    let mut f = tempfile::NamedTempFile::new_in(&dir)
        .with_context(|| format!("Failed to create a NamedTempFile in {dir:?}"))?;
    f.write_all(content)
        .with_context(|| format!("Failed to write the NamedTempFile at {:?}", f.path()))?;
    dfx_core::fs::rename(f.path(), path)?;
    Ok(())
}

#[context("Failed to copy candid path of pull dependency {name}")]
pub fn copy_service_candid_to_project(
    project_root: &Path,
    name: &str,
    canister_id: Principal,
) -> DfxResult {
    let service_candid_path = get_pulled_service_candid_path(canister_id)?;
    let path_in_project = get_candid_path_in_project(project_root, name);
    ensure_parent_dir_exists(&path_in_project)?;
    dfx_core::fs::copy(&service_candid_path, &path_in_project)?;
    Ok(())
}

fn parse_dfx_deps(deps_str: &str) -> DfxResult<Vec<Principal>> {
    let mut deps = vec![];
    for entry in deps_str.split_terminator(';') {
        match entry.split_once(':') {
            Some((_, p)) => {
                let dep_id = Principal::from_text(p)
                    .with_context(|| format!("`{p}` is not a valid Principal."))?;
                deps.push(dep_id);
            }
            None => bail!("Failed to parse `dfx:deps` entry: {entry}. Expected `name:Principal`. "),
        }
    }
    Ok(deps)
}
