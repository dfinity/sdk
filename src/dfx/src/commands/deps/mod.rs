use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;

use crate::lib::agent::create_agent_environment;
use crate::lib::deps::{InitJson, PulledJson};
use crate::lib::{environment::Environment, error::DfxResult};
use crate::NetworkOpt;

use anyhow::{anyhow, bail, Context};
use candid::Principal;
use clap::Parser;
use dfx_core::config::cache::get_cache_root;
use fn_error_context::context;
use sha2::{Digest, Sha256};
use tokio::runtime::Runtime;

mod init;
mod install;
mod pull;

/// Options for `dfx deps`.
#[derive(Parser)]
#[clap(name = "deps")]
pub struct DepsOpts {
    #[clap(flatten)]
    network: NetworkOpt,

    /// Arguments and flags for subcommands.
    #[clap(subcommand)]
    subcmd: SubCommand,
}

/// Subcommands of `dfx deps`
#[derive(Parser)]
enum SubCommand {
    Pull(pull::DepsPullOpts),
    Init(init::DepsInitOpts),
    Install(install::DepsInstallOpts),
}

/// Executes `dfx deps` and its subcommands.
pub fn exec(env: &dyn Environment, opts: DepsOpts) -> DfxResult {
    let agent_env = create_agent_environment(env, opts.network.network)?;
    let runtime = Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(async {
        match opts.subcmd {
            SubCommand::Pull(v) => pull::exec(&agent_env, v).await,
            SubCommand::Init(v) => init::exec(&agent_env, v).await,
            SubCommand::Install(v) => install::exec(&agent_env, v).await,
        }
    })
}

#[context("Failed to get pull canisters defined in dfx.json.")]
fn get_pull_canisters_in_config(env: &dyn Environment) -> DfxResult<BTreeMap<String, Principal>> {
    Ok(env
        .get_config_or_anyhow()?
        .get_config()
        .get_pull_canisters()?)
}

// 1. whether pulled.json is consistent with dfx.json
// 2. whether downloaded wasm modules are consistent with pulled.json
#[context("Failed to validate pulled.json and pulled Wasm modules. Please rerun `dfx deps pull`.")]
fn validate_pulled(
    pulled_json: &PulledJson,
    pull_canisters_in_config: &BTreeMap<String, Principal>,
) -> DfxResult {
    if &pulled_json.named != pull_canisters_in_config {
        bail!("The named section in pulled.json is not consistent with pull types canisters in dfx.json.");
    }

    for (canister_id, pulled_canister) in &pulled_json.canisters {
        let pulled_canister_path = get_pulled_wasm_path(*canister_id)?;
        let bytes = std::fs::read(pulled_canister_path)?;
        let hash_cache = Sha256::digest(bytes);
        let hash_in_json = hex::decode(&pulled_canister.wasm_hash)?;
        if hash_cache.as_slice() != hash_in_json {
            bail!("The pulled canister Wasm module has different hash than in pulled.json.");
        }
    }

    Ok(())
}

#[context("Failed to read pulled.json. Please (re)run `dfx deps pull`.")]
fn read_pulled_json(env: &dyn Environment) -> DfxResult<PulledJson> {
    let pulled_json_path = get_pulled_json_path(env)?;
    let pulled_json_str = std::fs::read_to_string(pulled_json_path)?;
    let pulled_json: PulledJson = serde_json::from_str::<PulledJson>(&pulled_json_str)?;
    Ok(pulled_json)
}

#[context("Failed to write pulled.json")]
fn write_pulled_json(env: &dyn Environment, pulled_json: &PulledJson) -> DfxResult {
    let pulled_json_path = get_pulled_json_path(env)?;
    let content = serde_json::to_string_pretty(pulled_json)?;
    write_to_tempfile_then_rename(content.as_bytes(), &pulled_json_path)?;
    Ok(())
}

#[context("Failed to get the path of pulled.json")]
fn get_pulled_json_path(env: &dyn Environment) -> DfxResult<PathBuf> {
    let project_root = env.get_config_or_anyhow()?.get_project_root().to_path_buf();
    Ok(project_root.join("deps").join("pulled.json"))
}

#[context("Failed to read init.json")]
fn read_init_json(env: &dyn Environment) -> DfxResult<InitJson> {
    let init_json_path = get_init_json_path(env)?;
    let init_json_str = std::fs::read_to_string(init_json_path)?;
    let init_json: InitJson = serde_json::from_str::<InitJson>(&init_json_str)?;
    Ok(init_json)
}

#[context("Failed to write init.json")]
fn write_init_json(env: &dyn Environment, init_json: &InitJson) -> DfxResult {
    let init_json_path = get_init_json_path(env)?;
    let content = serde_json::to_string_pretty(init_json)?;
    write_to_tempfile_then_rename(content.as_bytes(), &init_json_path)?;
    Ok(())
}

#[context("Failed to get the path of init.json")]
fn get_init_json_path(env: &dyn Environment) -> DfxResult<PathBuf> {
    let project_root = env.get_config_or_anyhow()?.get_project_root().to_path_buf();
    Ok(project_root.join("deps").join("init.json"))
}

#[context("Failed to get the path of pulled canister \"{canister_id}\"")]
fn get_pulled_wasm_path(canister_id: Principal) -> DfxResult<PathBuf> {
    Ok(get_cache_root()?
        .join("pulled")
        .join(canister_id.to_text())
        .join("canister.wasm"))
}

#[context("Failed to get the path of pulled canister \"{canister_id}\"")]
fn get_pulled_candid_path(canister_id: Principal) -> DfxResult<PathBuf> {
    Ok(get_cache_root()?
        .join("pulled")
        .join(canister_id.to_text())
        .join("canister.did"))
}

#[context("Failed to write to a tempfile then rename it to {}", path.display())]
fn write_to_tempfile_then_rename(content: &[u8], path: &PathBuf) -> DfxResult<()> {
    assert!(path.is_absolute());
    let dir = path
        .parent()
        .ok_or_else(|| anyhow!("Failed to get the parent dir from path"))?;
    std::fs::create_dir_all(dir).with_context(|| format!("Failed to create dir at {:?}", &dir))?;
    let mut f = tempfile::NamedTempFile::new_in(dir)?;
    f.write_all(content)?;
    std::fs::rename(f.path(), path)?;
    Ok(())
}
