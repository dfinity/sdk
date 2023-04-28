use crate::lib::agent::create_anonymous_agent_environment;
use crate::lib::deps::{
    get_candid_path_in_project, get_pull_canisters_in_config, get_pulled_service_candid_path,
    get_pulled_wasm_path, save_pulled_json,
};
use crate::lib::deps::{PulledCanister, PulledJson};
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::metadata::names::{
    CANDID_ARGS, CANDID_SERVICE, DFX_DEPS, DFX_INIT, DFX_WASM_HASH, DFX_WASM_URL,
};
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::state_tree::canister_info::read_state_tree_canister_module_hash;
use crate::lib::wasm::file::read_wasm_module;
use crate::util::download_file;
use crate::NetworkOpt;
use dfx_core::config::cache::get_cache_root;
use dfx_core::fs::composite::{ensure_dir_exists, ensure_parent_dir_exists};

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::io::Write;
use std::path::Path;

use anyhow::{anyhow, bail, Context};
use candid::Principal;
use clap::Parser;
use fn_error_context::context;
use ic_agent::{Agent, AgentError};
use ic_wasm::metadata::get_metadata;
use sha2::{Digest, Sha256};
use slog::{error, info, trace, warn, Logger};

/// Pull canisters upon which the project depends.
/// This command connects to the "ic" mainnet by default.
/// You can still choose other network by setting `--network`.
#[derive(Parser)]
pub struct DepsPullOpts {
    #[clap(flatten)]
    network: NetworkOpt,
}

pub async fn exec(env: &dyn Environment, opts: DepsPullOpts) -> DfxResult {
    let logger = env.get_logger();
    let pull_canisters_in_config = get_pull_canisters_in_config(env)?;
    if pull_canisters_in_config.is_empty() {
        info!(logger, "There are no pull dependencies defined in dfx.json");
        return Ok(());
    }

    let network = opts.network.network.unwrap_or_else(|| "ic".to_string());
    let env = create_anonymous_agent_environment(env, Some(network))?;

    let project_root = env.get_config_or_anyhow()?.get_project_root().to_path_buf();

    fetch_root_key_if_needed(&env).await?;

    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

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

async fn resolve_all_dependencies(
    agent: &Agent,
    logger: &Logger,
    pull_canisters_in_config: &BTreeMap<String, Principal>,
) -> DfxResult<Vec<Principal>> {
    let mut canisters_to_resolve: VecDeque<Principal> =
        pull_canisters_in_config.values().cloned().collect();
    let mut checked = BTreeSet::new();
    while let Some(canister_id) = canisters_to_resolve.pop_front() {
        if !checked.contains(&canister_id) {
            checked.insert(canister_id);
            let dependencies = get_dependencies(agent, logger, &canister_id).await?;
            canisters_to_resolve.extend(dependencies.iter());
        }
    }
    let all_dependencies = checked.into_iter().collect::<Vec<_>>();
    let mut message = String::new();
    message.push_str(&format!("Found {} dependencies:", all_dependencies.len()));
    for id in &all_dependencies {
        message.push('\n');
        message.push_str(&id.to_text());
    }
    info!(logger, "{}", message);
    Ok(all_dependencies)
}

#[context("Failed to get dependencies of canister {canister_id}.")]
async fn get_dependencies(
    agent: &Agent,
    logger: &Logger,
    canister_id: &Principal,
) -> DfxResult<Vec<Principal>> {
    info!(
        logger,
        "Resolving dependencies of canister {canister_id}..."
    );
    match fetch_metadata(agent, canister_id, DFX_DEPS).await? {
        Some(deps_raw) => {
            let deps_str = String::from_utf8(deps_raw)?;
            let deps = parse_dfx_deps(&deps_str)?;
            Ok(deps)
        }
        None => {
            warn!(
                logger,
                "`{DFX_DEPS}` metadata not found in canister {canister_id}."
            );
            Ok(vec![])
        }
    }
}

async fn download_all_and_generate_pulled_json(
    agent: &Agent,
    logger: &Logger,
    all_dependencies: &[Principal],
) -> DfxResult<PulledJson> {
    let mut any_download_fail = false;
    let mut pulled_json = PulledJson::default();
    for canister_id in all_dependencies {
        match download_and_generate_pulled_canister(agent, logger, *canister_id).await {
            Ok(pulled_canister) => {
                pulled_json.canisters.insert(*canister_id, pulled_canister);
            }
            Err(e) => {
                error!(logger, "Failed to pull canister {canister_id}.\n{e}");
                any_download_fail = true;
            }
        }
    }

    if any_download_fail {
        bail!("Failed when pulling canisters.");
    }
    Ok(pulled_json)
}

// Download canister wasm, then extract metadata from it to build a PulledCanister
async fn download_and_generate_pulled_canister(
    agent: &Agent,
    logger: &Logger,
    canister_id: Principal,
) -> DfxResult<PulledCanister> {
    info!(logger, "Pulling canister {canister_id}...");

    let mut pulled_canister = PulledCanister::default();

    // try fetch `dfx:wasm_hash`. If not available, get the hash of the on chain canister.
    let hash_on_chain = match fetch_metadata(agent, &canister_id, DFX_WASM_HASH).await? {
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

    let wasm_path = get_pulled_wasm_path(&canister_id)?;

    // skip download if cache hit
    let mut cache_hit = false;
    if wasm_path.exists() {
        let bytes = dfx_core::fs::read(&wasm_path)?;
        let hash_cache = Sha256::digest(bytes);
        if hash_cache.as_slice() == hash_on_chain {
            cache_hit = true;
            trace!(logger, "The canister wasm was found in the cache.");
        }
    }
    if !cache_hit {
        // fetch `dfx:wasm_url`
        let wasm_url_raw = fetch_metadata(agent, &canister_id, DFX_WASM_URL)
            .await?
            .ok_or_else(|| {
                anyhow!("`{DFX_WASM_URL}` metadata not found in canister {canister_id}.")
            })?;
        let wasm_url_str = String::from_utf8(wasm_url_raw)?;
        let wasm_url = reqwest::Url::parse(&wasm_url_str)?;

        // download
        let content = download_file(&wasm_url).await?;

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

    // extract `candid:service` and save as candid file in shared cache
    let module = read_wasm_module(&wasm_path)?;
    let candid_service = get_metadata_as_string(&module, CANDID_SERVICE, &wasm_path)?;
    let service_candid_path = get_pulled_service_candid_path(&canister_id)?;
    write_to_tempfile_then_rename(candid_service.as_bytes(), &service_candid_path)?;

    // extract `candid:args`
    let candid_args = get_metadata_as_string(&module, CANDID_ARGS, &wasm_path)?;
    pulled_canister.candid_args = Some(candid_args);

    // try extract `dfx:deps`
    if let Ok(dfx_deps) = get_metadata_as_string(&module, DFX_DEPS, &wasm_path) {
        let deps = parse_dfx_deps(&dfx_deps)?;
        pulled_canister.deps = deps;
    } else {
        trace!(
            logger,
            "{:?} doesn't define {} metadata",
            &wasm_path,
            DFX_DEPS
        );
    }

    // try extract `dfx:init`
    if let Ok(dfx_init) = get_metadata_as_string(&module, DFX_INIT, &wasm_path) {
        pulled_canister.dfx_init = Some(dfx_init)
    } else {
        trace!(
            logger,
            "{:?} doesn't define {} metadata",
            &wasm_path,
            DFX_INIT
        );
    }

    Ok(pulled_canister)
}

#[context("Failed to fetch metadata {metadata} of canister {canister_id}.")]
async fn fetch_metadata(
    agent: &Agent,
    canister_id: &Principal,
    metadata: &str,
) -> DfxResult<Option<Vec<u8>>> {
    match agent
        .read_state_canister_metadata(*canister_id, metadata)
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
    canister_id: &Principal,
) -> DfxResult {
    let service_candid_path = get_pulled_service_candid_path(canister_id)?;
    let path_in_project = get_candid_path_in_project(project_root, canister_id);
    ensure_parent_dir_exists(&path_in_project)?;
    dfx_core::fs::copy(&service_candid_path, &path_in_project)?;
    Ok(())
}

fn parse_dfx_deps(deps_str: &str) -> DfxResult<Vec<Principal>> {
    let mut deps = vec![];
    for entry in deps_str.split_terminator(';') {
        let canister_id = Principal::from_text(entry).with_context(|| {
            format!("Found invalid entry in `dfx:deps`: \"{entry}\". Expected a Principal.")
        })?;
        deps.push(canister_id);
    }
    Ok(deps)
}

fn get_metadata_as_string(
    module: &walrus::Module,
    section: &str,
    wasm_path: &Path,
) -> DfxResult<String> {
    let metadata_bytes = get_metadata(module, section)
        .with_context(|| format!("Failed to get {} metadata from {:?}", section, wasm_path))?;
    let metadata = String::from_utf8(metadata_bytes.to_vec()).with_context(|| {
        format!(
            "Failed to read {} metadata from {:?} as UTF-8 text",
            section, wasm_path
        )
    })?;
    Ok(metadata)
}
