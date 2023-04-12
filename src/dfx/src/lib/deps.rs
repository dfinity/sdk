use std::{collections::BTreeMap, path::PathBuf};

use anyhow::{bail, Context};
use candid::Principal;
use dfx_core::{json::{save_json_file, load_json_file}, config::cache::get_cache_root};
use fn_error_context::context;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

use super::{error::DfxResult, environment::Environment};

#[derive(Serialize, Deserialize, Default)]
pub struct PulledJson {
    pub named: BTreeMap<String, Principal>,
    pub canisters: BTreeMap<Principal, PulledCanister>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct PulledCanister {
    // dfx:deps
    pub deps: Vec<Principal>,
    // dfx:wasm_url, once we can download wasm directly from IC, this field will be optional
    pub wasm_url: Option<String>,
    // the hash on chain
    // dfx:wasm_hash if defined
    // or get from canister_status
    pub wasm_hash: String,
    // dfx:init
    pub init: Option<String>,
}

impl PulledJson {
    pub fn with_named(named: &BTreeMap<String, Principal>) -> Self {
        Self {
            named: named.clone(),
            ..Default::default()
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct InitJson {
    canisters: BTreeMap<Principal, InitItem>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct InitItem {
    // init argument in IDL string
    arg_str: Option<String>,
    // hex encoded bytes of init argument
    arg_raw: Option<String>,
}

impl InitJson {
    pub fn set_init_arg(
        &mut self,
        canister_id: Principal,
        arg_str: Option<String>,
        arg_raw: &[u8],
    ) {
        self.canisters.insert(
            canister_id,
            InitItem {
                arg_str,
                arg_raw: Some(hex::encode(arg_raw)),
            },
        );
    }

    pub fn set_empty_init(&mut self, canister_id: Principal) {
        self.canisters.insert(canister_id, InitItem::default());
    }

    pub fn contains(&self, canister_id: &Principal) -> bool {
        self.canisters.contains_key(canister_id)
    }

    pub fn get_arg_raw(&self, canister_id: &Principal) -> DfxResult<Vec<u8>> {
        match self.canisters.get(canister_id) {
            Some(item) => match &item.arg_raw {
                Some(s) => Ok(hex::decode(s)?),
                None => Ok(vec![]),
            },
            None => bail!(
                "Failed to find {0} entry in init.json. Please run `dfx deps init {0}`.",
                canister_id
            ),
        }
    }
}


#[context("Failed to get pull canisters defined in dfx.json.")]
pub fn get_pull_canisters_in_config(env: &dyn Environment) -> DfxResult<BTreeMap<String, Principal>> {
    Ok(env
        .get_config_or_anyhow()?
        .get_config()
        .get_pull_canisters()?)
}

// 1. whether pulled.json is consistent with dfx.json
// 2. whether downloaded wasm modules are consistent with pulled.json
#[context("Failed to validate pulled.json and pulled Wasm modules. Please rerun `dfx deps pull`.")]
pub fn validate_pulled(
    pulled_json: &PulledJson,
    pull_canisters_in_config: &BTreeMap<String, Principal>,
) -> DfxResult {
    if &pulled_json.named != pull_canisters_in_config {
        bail!("The named section in pulled.json is not consistent with pull types canisters in dfx.json.");
    }

    for (canister_id, pulled_canister) in &pulled_json.canisters {
        let pulled_canister_path = get_pulled_wasm_path(*canister_id)?;
        let bytes = std::fs::read(&pulled_canister_path)
            .context(format!("Failed to read {:?}", &pulled_canister_path))?;
        let hash_cache = Sha256::digest(bytes);
        let hash_in_json = hex::decode(&pulled_canister.wasm_hash)?;
        if hash_cache.as_slice() != hash_in_json {
            bail!("The pulled canister Wasm module has different hash than in pulled.json.");
        }
    }

    Ok(())
}

fn get_deps_dir(env: &dyn Environment) -> DfxResult<PathBuf> {
    let project_root = env.get_config_or_anyhow()?.get_project_root().to_path_buf();
    Ok(project_root.join("deps"))
}

fn get_init_json_path(env: &dyn Environment) -> DfxResult<PathBuf> {
    Ok(get_deps_dir(env)?.join("init.json"))
}

fn get_pulled_json_path(env: &dyn Environment) -> DfxResult<PathBuf> {
    Ok(get_deps_dir(env)?.join("pulled.json"))
}

#[context("Failed to read pulled.json. Please (re)run `dfx deps pull`.")]
pub fn load_pulled_json(env: &dyn Environment) -> DfxResult<PulledJson> {
    let pulled_json_path = get_pulled_json_path(env)?;
    let pulled_json = load_json_file(&pulled_json_path)?;
    Ok(pulled_json)
}

#[context("Failed to write pulled.json")]
pub fn save_pulled_json(env: &dyn Environment, pulled_json: &PulledJson) -> DfxResult {
    let pulled_json_path = get_pulled_json_path(env)?;
    save_json_file(&pulled_json_path, pulled_json)?;
    Ok(())
}

#[context("Failed to create init.json")]
pub fn create_init_json_if_not_existed(env: &dyn Environment) -> DfxResult {
    let init_json_path = get_init_json_path(env)?;
    if !init_json_path.exists() {
        let init_json = InitJson::default();
        save_json_file(&init_json_path, &init_json)?;
    }
    Ok(())
}

#[context("Failed to read init.json")]
pub fn load_init_json(env: &dyn Environment) -> DfxResult<InitJson> {
    let init_json_path = get_init_json_path(env)?;
    let init_json = load_json_file(&init_json_path)?;
    Ok(init_json)
}

#[context("Failed to write init.json")]
pub fn save_init_json(env: &dyn Environment, init_json: &InitJson) -> DfxResult {
    let init_json_path = get_init_json_path(env)?;
    save_json_file(&init_json_path, init_json)?;
    Ok(())
}

#[context("Failed to get the path of pulled canister \"{canister_id}\"")]
pub fn get_pulled_wasm_path(canister_id: Principal) -> DfxResult<PathBuf> {
    Ok(get_cache_root()?
        .join("pulled")
        .join(canister_id.to_text())
        .join("canister.wasm"))
}

#[context("Failed to get the service candid path of pulled canister \"{canister_id}\"")]
pub fn get_service_candid_path(canister_id: Principal) -> DfxResult<PathBuf> {
    Ok(get_cache_root()?
        .join("pulled")
        .join(canister_id.to_text())
        .join("service.did"))
}

#[context("Failed to copy candid path of pull dependency {name}")]
pub fn copy_service_candid_to_project(
    env: &dyn Environment,
    name: &str,
    canister_id: Principal,
) -> DfxResult {
    let service_candid_path = get_service_candid_path(canister_id)?;
    let deps_dir = get_deps_dir(env)?;
    std::fs::create_dir_all(&deps_dir)
        .with_context(|| format!("Failed to create dir at {:?}", &deps_dir))?;
    let path_in_project = deps_dir.join(format!("{name}.did"));
    std::fs::copy(&service_candid_path, &path_in_project)?;
    Ok(())
}
