use crate::lib::{environment::Environment, error::DfxResult};
use dfx_core::{
    config::cache::get_cache_root,
    fs::composite::ensure_parent_dir_exists,
    json::{load_json_file, save_json_file},
};

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail, Context};
use candid::Principal;
use fn_error_context::context;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize, Default)]
pub struct PulledJson {
    pub canisters: BTreeMap<Principal, PulledCanister>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct PulledCanister {
    // name of `type: pull` in dfx.json. None if indirect dependency.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    // from the dfx metadata of the downloaded wasm module
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub dependencies: Vec<Principal>,
    // the hash of the canister wasm on chain
    // wasm_hash if defined in the dfx metadata
    // or get from canister_status
    pub wasm_hash: String,
    // from the dfx metadata of the downloaded wasm module
    pub init_guide: String,
    // from the candid:args metadata of the downloaded wasm module
    pub candid_args: String,
}

impl PulledJson {
    pub fn get_init(&self, canister_id: &Principal) -> DfxResult<&str> {
        match self.canisters.get(canister_id) {
            Some(o) => Ok(&o.init_guide),
            None => bail!("Failed to find {canister_id} in pulled.json"),
        }
    }

    pub fn get_candid_args(&self, canister_id: &Principal) -> DfxResult<&str> {
        let pulled_canister = self
            .canisters
            .get(canister_id)
            .ok_or_else(|| anyhow!("Failed to find {canister_id} in pulled.json"))?;
        Ok(&pulled_canister.candid_args)
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
        canister_id: &Principal,
        arg_str: Option<String>,
        arg_raw: &[u8],
    ) {
        self.canisters.insert(
            *canister_id,
            InitItem {
                arg_str,
                arg_raw: Some(hex::encode(arg_raw)),
            },
        );
    }

    pub fn set_empty_init(&mut self, canister_id: &Principal) {
        self.canisters.insert(*canister_id, InitItem::default());
    }

    pub fn contains(&self, canister_id: &Principal) -> bool {
        self.canisters.contains_key(canister_id)
    }

    pub fn get_arg_raw(&self, canister_id: &Principal) -> DfxResult<Vec<u8>> {
        let init_item = self.canisters.get(canister_id).ok_or_else(|| {
            anyhow!(
                "Failed to find {0} entry in init.json. Please run `dfx deps init {0}`.",
                canister_id
            )
        })?;
        match &init_item.arg_raw {
            Some(s) => Ok(hex::decode(s)?),
            None => Ok(vec![]),
        }
    }
}

#[context("Failed to get pull canisters defined in dfx.json.")]
pub fn get_pull_canisters_in_config(
    env: &dyn Environment,
) -> DfxResult<BTreeMap<String, Principal>> {
    Ok(env
        .get_config_or_anyhow()?
        .get_config()
        .get_pull_canisters()?)
}

// 1. whether pulled.json is consistent with dfx.json
// 2. whether downloaded wasm modules are consistent with pulled.json
pub fn validate_pulled(
    pulled_json: &PulledJson,
    pull_canisters_in_config: &BTreeMap<String, Principal>,
) -> DfxResult {
    for (name, canister_id) in pull_canisters_in_config {
        let pulled_canister = pulled_json
            .canisters
            .get(canister_id)
            .ok_or_else(|| anyhow!("Failed to find {name}:{canister_id} in pulled.json."))?;
        match &pulled_canister.name {
            Some(s) if s == name => continue,
            Some(other_name) => bail!(
                "{canister_id} is \"{name}\" in dfx.json, but it has name \"{}\" in pulled.json.",
                other_name
            ),
            None => bail!(
                "{canister_id} is \"{name}\" in dfx.json, but it doesn't have name in pulled.json."
            ),
        }
    }

    for (canister_id, pulled_canister) in &pulled_json.canisters {
        let pulled_canister_path = get_pulled_wasm_path(canister_id)?;
        let bytes = dfx_core::fs::read(&pulled_canister_path)?;
        let hash_cache = Sha256::digest(bytes);
        let hash_in_json = hex::decode(&pulled_canister.wasm_hash)?;
        if hash_cache.as_slice() != hash_in_json {
            let hash_cache = hex::encode(hash_cache.as_slice());
            let hash_in_json = &pulled_canister.wasm_hash;
            bail!(
                "The pulled wasm of {canister_id} has different hash than in pulled.json:
    The pulled wasm is at {pulled_canister_path:?}. Its hash is:
        {hash_cache}
    The expected hash in pulled.json is:
        {hash_in_json}"
            );
        }
    }

    Ok(())
}

fn get_deps_dir(project_root: &Path) -> PathBuf {
    project_root.join("deps")
}

pub fn get_candid_path_in_project(project_root: &Path, canister_id: &Principal) -> PathBuf {
    get_deps_dir(project_root)
        .join("candid")
        .join(canister_id.to_text())
        .with_extension("did")
}

fn get_init_json_path(project_root: &Path) -> PathBuf {
    get_deps_dir(project_root).join("init.json")
}

fn get_pulled_json_path(project_root: &Path) -> PathBuf {
    get_deps_dir(project_root).join("pulled.json")
}

#[context("Failed to read pulled.json. Please (re)run `dfx deps pull`.")]
pub fn load_pulled_json(project_root: &Path) -> DfxResult<PulledJson> {
    let pulled_json_path = get_pulled_json_path(project_root);
    let pulled_json = load_json_file(&pulled_json_path)?;
    Ok(pulled_json)
}

#[context("Failed to save pulled.json")]
pub fn save_pulled_json(project_root: &Path, pulled_json: &PulledJson) -> DfxResult {
    let pulled_json_path = get_pulled_json_path(project_root);
    ensure_parent_dir_exists(&pulled_json_path)?;
    save_json_file(&pulled_json_path, pulled_json)?;
    Ok(())
}

#[context("Failed to create init.json")]
pub fn create_init_json_if_not_existed(project_root: &Path) -> DfxResult {
    let init_json_path = get_init_json_path(project_root);
    if !init_json_path.exists() {
        let init_json = InitJson::default();
        ensure_parent_dir_exists(&init_json_path)?;
        save_json_file(&init_json_path, &init_json)?;
    }
    Ok(())
}

#[context("Failed to read init.json. Please run `dfx deps init`.")]
pub fn load_init_json(project_root: &Path) -> DfxResult<InitJson> {
    let init_json_path = get_init_json_path(project_root);
    let init_json = load_json_file(&init_json_path)?;
    Ok(init_json)
}

#[context("Failed to save init.json")]
pub fn save_init_json(project_root: &Path, init_json: &InitJson) -> DfxResult {
    let init_json_path = get_init_json_path(project_root);
    ensure_parent_dir_exists(&init_json_path)?;
    save_json_file(&init_json_path, init_json)?;
    Ok(())
}

#[context("Failed to get the wasm path of pulled canister \"{canister_id}\"")]
pub fn get_pulled_wasm_path(canister_id: &Principal) -> DfxResult<PathBuf> {
    Ok(get_pulled_canister_dir(canister_id)?.join("canister.wasm"))
}

#[context("Failed to get the service candid path of pulled canister \"{canister_id}\"")]
pub fn get_pulled_service_candid_path(canister_id: &Principal) -> DfxResult<PathBuf> {
    Ok(get_pulled_canister_dir(canister_id)?.join("service.did"))
}

fn get_pulled_canister_dir(canister_id: &Principal) -> DfxResult<PathBuf> {
    Ok(get_cache_root()?.join("pulled").join(canister_id.to_text()))
}

pub fn get_pull_canister_or_principal(
    canister: &str,
    pull_canisters_in_config: &BTreeMap<String, Principal>,
    pulled_json: &PulledJson,
) -> DfxResult<Principal> {
    match pull_canisters_in_config.get(canister) {
        Some(canister_id) => Ok(*canister_id),
        None => {
            let p = Principal::from_text(canister).with_context(||
                format!("{canister} is not a valid Principal nor a `type: pull` canister specified in dfx.json")
            )?;
            if pulled_json.canisters.get(&p).is_none() {
                bail!("Could not find {} in pulled.json", &p);
            }
            Ok(p)
        }
    }
}

pub fn get_canister_prompt(canister_id: &Principal, pulled_canister: &PulledCanister) -> String {
    match &pulled_canister.name {
        Some(name) => format!("{canister_id} ({name})"),
        None => canister_id.to_text(),
    }
}
