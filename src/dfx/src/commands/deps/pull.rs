use crate::lib::agent::create_anonymous_agent_environment;
use crate::lib::deps::{
    get_candid_path_in_project, get_pull_canisters_in_config, get_pulled_canister_dir,
    get_pulled_service_candid_path, get_pulled_wasm_path, save_pulled_json,
};
use crate::lib::deps::{PulledCanister, PulledJson};
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::metadata::dfx::DfxMetadata;
use crate::lib::metadata::names::{CANDID_ARGS, CANDID_SERVICE, DFX};
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::state_tree::canister_info::read_state_tree_canister_module_hash;
use crate::lib::wasm::file::{decompress_bytes, read_wasm_module};
use crate::util::download_file;
use crate::NetworkOpt;
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
use slog::{error, info, trace, Logger};

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
    info!(logger, "Fetching dependencies of canister {canister_id}...");
    let dfx_metadata = fetch_dfx_metadata(agent, canister_id).await?;
    let dependencies = dfx_metadata.get_pullable()?.dependencies.clone();
    Ok(dependencies)
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

    let dfx_metadata = fetch_dfx_metadata(agent, &canister_id).await?;
    let pullable = dfx_metadata.get_pullable()?;

    // lookup `wasm_hash` in dfx metadata. If not available, get the hash of the on chain canister.
    let hash_on_chain = match &pullable.wasm_hash {
        Some(wasm_hash_str) => {
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

    // skip download if cache hit
    let mut cache_hit = false;

    for gzip in [false, true] {
        let path = get_pulled_wasm_path(&canister_id, gzip)?;
        if path.exists() {
            let bytes = dfx_core::fs::read(&path)?;
            let hash_cache = Sha256::digest(bytes);
            if hash_cache.as_slice() == hash_on_chain {
                cache_hit = true;
                pulled_canister.gzip = gzip;
                trace!(logger, "The canister wasm was found in the cache.");
            }
            break;
        }
    }

    if !cache_hit {
        // delete files from previous pull
        let pulled_canister_dir = get_pulled_canister_dir(&canister_id)?;
        if pulled_canister_dir.exists() {
            dfx_core::fs::remove_dir_all(&pulled_canister_dir)?;
        }
        dfx_core::fs::create_dir_all(&pulled_canister_dir)?;

        // lookup `wasm_url` in dfx metadata
        let wasm_url = reqwest::Url::parse(&pullable.wasm_url)?;

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

        let gzip = decompress_bytes(&content).is_ok();
        pulled_canister.gzip = gzip;
        let wasm_path = get_pulled_wasm_path(&canister_id, gzip)?;

        write_to_tempfile_then_rename(&content, &wasm_path)?;
    }

    let wasm_path = get_pulled_wasm_path(&canister_id, pulled_canister.gzip)?;

    // extract `candid:service` and save as candid file in shared cache
    let module = read_wasm_module(&wasm_path)?;
    let candid_service = get_metadata_as_string(&module, CANDID_SERVICE, &wasm_path)?;
    let service_candid_path = get_pulled_service_candid_path(&canister_id)?;
    write_to_tempfile_then_rename(candid_service.as_bytes(), &service_candid_path)?;

    // extract `candid:args`
    let candid_args = get_metadata_as_string(&module, CANDID_ARGS, &wasm_path)?;
    pulled_canister.candid_args = candid_args;

    // extract `dfx`
    let dfx_metadata_str = get_metadata_as_string(&module, DFX, &wasm_path)?;
    let dfx_metadata: DfxMetadata = serde_json::from_str(&dfx_metadata_str)?;
    let pullable = dfx_metadata.get_pullable()?;
    pulled_canister.dependencies = pullable.dependencies.clone();
    pulled_canister.init_guide = pullable.init_guide.clone();

    Ok(pulled_canister)
}

async fn fetch_dfx_metadata(agent: &Agent, canister_id: &Principal) -> DfxResult<DfxMetadata> {
    match fetch_metadata(agent, canister_id, DFX).await? {
        Some(dfx_metadata_raw) => {
            let dfx_metadata_str = String::from_utf8(dfx_metadata_raw)?;
            let dfx_metadata: DfxMetadata = serde_json::from_str(&dfx_metadata_str)?;
            Ok(dfx_metadata)
        }
        None => {
            bail!("`{DFX}` metadata not found in canister {canister_id}.");
        }
    }
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
