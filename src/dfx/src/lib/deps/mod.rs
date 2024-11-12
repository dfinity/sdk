use crate::lib::{environment::Environment, error::DfxResult};
use anyhow::{anyhow, bail, Context};
use candid::Principal;
use dfx_core::{
    config::cache::get_cache_root,
    fs::composite::ensure_parent_dir_exists,
    json::{load_json_file, save_json_file},
};
use fn_error_context::context;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

pub mod deploy;
pub mod pull;

#[derive(Serialize, Deserialize, Default)]
pub struct PulledJson {
    pub canisters: BTreeMap<Principal, PulledCanister>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct PulledCanister {
    /// Name of `type: pull` in dfx.json. Omitted if indirect dependency.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// From the dfx metadata of the downloaded wasm module
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub dependencies: Vec<Principal>,
    /// The expected module hash of the canister wasm
    /// Will be one of the following:
    ///   - wasm_hash if defined in the dfx metadata
    ///   - wasm_hash_url content if defined in the dfx metadata
    ///   - otherwise read from canister_status
    /// This field is kept here so that users can compare the hash of the downloaded wasm with it
    /// If matched, we get extra confidence that the downloaded wasm is correct
    /// If not matched, it is still acceptable
    pub wasm_hash: String,
    /// The downloaded wasm hash when `dfx deps pull`
    /// It is allowed to be different from `wasm_hash`
    pub wasm_hash_download: String,
    /// From the dfx metadata of the downloaded wasm module
    pub init_guide: String,
    /// From the dfx metadata of the downloaded wasm module
    pub init_arg: Option<String>,
    /// From the candid:args metadata of the downloaded wasm module
    pub candid_args: String,
    /// The downloaded wasm is gzip or not
    pub gzip: bool,
}

impl PulledJson {
    pub fn get_init_guide(&self, canister_id: &Principal) -> DfxResult<&str> {
        match self.canisters.get(canister_id) {
            Some(o) => Ok(&o.init_guide),
            None => bail!("Failed to find {canister_id} in pulled.json"),
        }
    }

    pub fn get_init_arg(&self, canister_id: &Principal) -> DfxResult<Option<&str>> {
        match self.canisters.get(canister_id) {
            Some(o) => Ok(o.init_arg.as_deref()),
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
    /// Init argument in IDL string
    arg_str: Option<String>,
    /// Hex encoded bytes of init argument
    arg_raw: Option<String>,
}

impl InitJson {
    /// Set `init_arg` for a pull dependency.
    ///
    /// The input `arg_str` is optional since users may specify the raw argument directly.
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

    /// Set empty `init_arg` for a pull dependency.
    pub fn set_empty_init(&mut self, canister_id: &Principal) {
        self.canisters.insert(*canister_id, InitItem::default());
    }

    /// Whether already set `init
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

/// Map from canister name to its Principal as defined in `dfx.json.
#[context("Failed to get pull canisters defined in dfx.json.")]
pub fn get_pull_canisters_in_config(
    env: &dyn Environment,
) -> DfxResult<BTreeMap<String, Principal>> {
    Ok(env
        .get_config_or_anyhow()?
        .get_config()
        .get_pull_canisters()?)
}

/// Validate following properties:
///   - whether `pulled.json` is consistent with `dfx.json`
///     - pull canisters in `dfx.json` are in `pulled.json` with the same name
///   - whether the wasm modules in pulled cache are consistent with `pulled.json`
///     - This can happen when the user manually modifies the wasm file in the cache
///     - Or the same canister was pulled in different projects and the downloaded wasm is different
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
        let pulled_canister_path = get_pulled_wasm_path(canister_id, pulled_canister.gzip)?;
        let bytes = dfx_core::fs::read(&pulled_canister_path)?;
        let hash_cache = Sha256::digest(bytes);
        let hash_in_json = hex::decode(&pulled_canister.wasm_hash_download)
            .with_context(|| format!{"In pulled.json, the `wasm_hash_download` field of {canister_id} is invalid."})?;
        if hash_cache.as_slice() != hash_in_json {
            let hash_cache = hex::encode(hash_cache.as_slice());
            let hash_in_json = &pulled_canister.wasm_hash_download;
            bail!(
                "The wasm of {canister_id} in pulled cache has different hash than in pulled.json:
    The pulled cache is at {pulled_canister_path:?}. Its hash is:
        {hash_cache}
    The hash (wasm_hash_download) in pulled.json is:
        {hash_in_json}
The pulled cache may be modified manually or the same canister was pulled in different projects."
            );
        }
    }

    Ok(())
}

fn get_deps_dir(project_root: &Path) -> PathBuf {
    project_root.join("deps")
}

/// The path of the candid file of a direct dependency.
///
/// `deps/candid/<PRINCIPAL>.did`.
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

/// Load `pulled.json` in `deps/`.
#[context("Failed to read pulled.json. Please (re)run `dfx deps pull`.")]
pub fn load_pulled_json(project_root: &Path) -> DfxResult<PulledJson> {
    let pulled_json_path = get_pulled_json_path(project_root);
    let pulled_json = load_json_file(&pulled_json_path)?;
    Ok(pulled_json)
}

/// Save `pulled.json` in `deps/`.
#[context("Failed to save pulled.json")]
pub fn save_pulled_json(project_root: &Path, pulled_json: &PulledJson) -> DfxResult {
    let pulled_json_path = get_pulled_json_path(project_root);
    ensure_parent_dir_exists(&pulled_json_path)?;
    save_json_file(&pulled_json_path, pulled_json)?;
    Ok(())
}

/// Create `init.json` in `deps/`.
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

/// Load the `init.json` in `deps/`.
#[context("Failed to read init.json. Please run `dfx deps init`.")]
pub fn load_init_json(project_root: &Path) -> DfxResult<InitJson> {
    let init_json_path = get_init_json_path(project_root);
    let init_json = load_json_file(&init_json_path)?;
    Ok(init_json)
}

/// Save `init.json` in `deps/`.
#[context("Failed to save init.json")]
pub fn save_init_json(project_root: &Path, init_json: &InitJson) -> DfxResult {
    let init_json_path = get_init_json_path(project_root);
    ensure_parent_dir_exists(&init_json_path)?;
    save_json_file(&init_json_path, init_json)?;
    Ok(())
}

/// The path of the downloaded .wasm or .wasm.gz file.
#[context("Failed to get the wasm path of pulled canister \"{canister_id}\"")]
pub fn get_pulled_wasm_path(canister_id: &Principal, gzip: bool) -> DfxResult<PathBuf> {
    let p = get_pulled_canister_dir(canister_id)?.join("canister");
    match gzip {
        true => Ok(p.with_extension("wasm.gz")),
        false => Ok(p.with_extension("wasm")),
    }
}

/// The path of service.did file extracted from the downloaded wasm.
#[context("Failed to get the service candid path of pulled canister \"{canister_id}\"")]
pub fn get_pulled_service_candid_path(canister_id: &Principal) -> DfxResult<PathBuf> {
    Ok(get_pulled_canister_dir(canister_id)?.join("service.did"))
}

/// The path of the dir contains wasm and service.did.
pub fn get_pulled_canister_dir(canister_id: &Principal) -> DfxResult<PathBuf> {
    let p = get_cache_root()?;
    Ok(p.join("pulled").join(canister_id.to_text()))
}

/// Get the principal of a pull dependency which must exist in `pulled.json`.
///
/// The input can be one of:
///   - <PRINCIPAL> of any pull dependency
///   - <NAME> of direct dependencies defined in `dfx.json`
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

/// The prompt of a pull dependency.
///
/// Will be one of the following:
///   - "<CANISTER_ID>" if it is not a direct dependency
///   - "<CANISTER_ID> (<NAME>)" if it is a direct dependency with NAME defined in `dfx.json`
pub fn get_canister_prompt(canister_id: &Principal, pulled_canister: &PulledCanister) -> String {
    match &pulled_canister.name {
        Some(name) => format!("{canister_id} ({name})"),
        None => canister_id.to_text(),
    }
}
