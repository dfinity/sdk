//! Named canister module.
//!
//! Maps named canister to canister id.
use crate::config::dfinity::NetworkType;
use crate::lib::config::get_config_dfx_dir_path;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::network::network_descriptor::NetworkDescriptor;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::waiter::waiter_with_timeout;
use crate::util;
use crate::util::expiry_duration;

use anyhow::anyhow;
use ic_types::Principal;
use ic_utils::call::AsyncCall;
use ic_utils::interfaces::management_canister::{InstallMode, MemoryAllocation};
use ic_utils::interfaces::{HttpRequestCanister, ManagementCanister};
use ic_utils::Canister;
use serde::{Deserialize, Serialize};
use slog::info;
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::io::Read;
use std::path::PathBuf;

const DEFAULT_MEM_ALLOCATION: u64 = 5000000_u64; // 5 MB
const CONFIG_FILE_NAME: &str = "canister.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct NamedCanister {
    pub name: String,
    #[serde(flatten)]
    pub networks: BTreeMap<String, Principal>,
}

impl NamedCanister {
    fn get_config_file(env: &dyn Environment, network: &NetworkDescriptor) -> DfxResult<PathBuf> {
        Ok(match network.r#type {
            NetworkType::Persistent => get_config_dfx_dir_path()?
                .join("canister")
                .join(CONFIG_FILE_NAME),
            NetworkType::Ephemeral => env.get_temp_dir().join("local").join(CONFIG_FILE_NAME),
        })
    }
    fn get_config(
        env: &dyn Environment,
        network: &NetworkDescriptor,
    ) -> DfxResult<(PathBuf, Self)> {
        let path = NamedCanister::get_config_file(env, network)?;
        Ok(if path.exists() {
            let mut buffer = Vec::new();
            std::fs::File::open(&path)?.read_to_end(&mut buffer)?;
            (path, serde_json::from_slice::<NamedCanister>(&buffer)?)
        } else {
            (
                path,
                NamedCanister {
                    name: "UI".to_string(),
                    networks: BTreeMap::new(),
                },
            )
        })
    }
    pub async fn install_ui_canister(
        env: &dyn Environment,
        network: &NetworkDescriptor,
        some_canister_id: Option<Principal>,
    ) -> DfxResult<Principal> {
        let (path, mut config) = Self::get_config(env, network)?;
        if config.networks.get(&network.name).is_some() {
            return Err(anyhow!(
                "{} canister already installed on {} network",
                config.name,
                network.name
            ));
        }
        fetch_root_key_if_needed(env).await?;
        let mgr = ManagementCanister::create(
            env.get_agent()
                .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?,
        );
        info!(
            env.get_logger(),
            "Creating {} canister on the {} network.", config.name, network.name
        );
        let mut canister_assets = util::assets::ui_canister()?;
        let mut wasm = Vec::new();
        for file in canister_assets.entries()? {
            let mut file = file?;
            if file.header().path()?.ends_with("ui.wasm") {
                file.read_to_end(&mut wasm)?;
            }
        }
        let canister_id = match some_canister_id {
            Some(id) => id,
            None => {
                if network.is_ic {
                    // Provisional commands are whitelisted on production
                    mgr.create_canister()
                        .call_and_wait(waiter_with_timeout(expiry_duration()))
                        .await?
                        .0
                } else {
                    mgr.provisional_create_canister_with_cycles(None)
                        .call_and_wait(waiter_with_timeout(expiry_duration()))
                        .await?
                        .0
                }
            }
        };
        mgr.install_code(&canister_id, wasm.as_slice())
            .with_mode(InstallMode::Install)
            .with_memory_allocation(
                MemoryAllocation::try_from(DEFAULT_MEM_ALLOCATION).expect(
                    "Memory allocation must be between 0 and 2^48 (i.e 256TB), inclusively.",
                ),
            )
            .call_and_wait(waiter_with_timeout(expiry_duration()))
            .await?;
        config
            .networks
            .insert(network.name.clone(), canister_id.clone());
        std::fs::create_dir_all(path.parent().unwrap())?;
        std::fs::write(&path, &serde_json::to_string_pretty(&config)?)?;
        Ok(canister_id)
    }
    pub fn get_canister_id(
        env: &dyn Environment,
        network: &NetworkDescriptor,
    ) -> DfxResult<Principal> {
        let config = Self::get_config(env, network)?.1;
        Ok(config
            .networks
            .get(&network.name)
            .ok_or_else(|| {
                anyhow!(
                    "Cannot find {} canister on network {}",
                    config.name,
                    network.name
                )
            })?
            .clone())
    }
    pub async fn get_ui_canister<'env>(
        env: &'env dyn Environment,
        network: &NetworkDescriptor,
    ) -> DfxResult<Canister<'env, HttpRequestCanister>> {
        let id = Self::get_canister_id(env, network)?;
        Ok(HttpRequestCanister::create(
            env.get_agent()
                .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?,
            id,
        ))
    }
}
