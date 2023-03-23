use crate::lib::environment::Environment;
use crate::lib::error::CanisterStoreIdError;
use crate::lib::network::directory::ensure_cohesive_network_directory;
use dfx_core::config::model::dfinity::Config;
use dfx_core::config::model::network_descriptor::{NetworkDescriptor, NetworkTypeDescriptor};

use candid::Principal as CanisterId;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

pub type CanisterName = String;
pub type NetworkName = String;
pub type CanisterIdString = String;

pub type NetworkNametoCanisterId = BTreeMap<NetworkName, CanisterIdString>;
pub type CanisterIds = BTreeMap<CanisterName, NetworkNametoCanisterId>;

#[derive(Clone, Debug)]
pub struct CanisterIdStore {
    network_descriptor: NetworkDescriptor,
    path: Option<PathBuf>,

    // Only the canister ids read from/written to canister-ids.json
    // which does not include remote canister ids
    ids: CanisterIds,

    // Remote ids read from dfx.json, never written to canister_ids.json
    remote_ids: Option<CanisterIds>,
}

impl CanisterIdStore {
    pub const DEFAULT: &'static str = "__default";

    pub fn for_env(env: &dyn Environment) -> Result<Self, CanisterStoreIdError> {
        CanisterIdStore::new(env.get_network_descriptor(), env.get_config()).map_err(|e| {
            CanisterStoreIdError::LoadCanisterIdStore {
                source: Box::new(e),
            }
        })
    }

    pub fn new(
        network_descriptor: &NetworkDescriptor,
        config: Option<Arc<Config>>,
    ) -> Result<Self, CanisterStoreIdError> {
        let path = match network_descriptor {
            NetworkDescriptor {
                r#type: NetworkTypeDescriptor::Persistent,
                ..
            } => config
                .as_ref()
                .map(|c| c.get_project_root().join("canister_ids.json")),
            NetworkDescriptor { name, .. } => match &config {
                None => None,
                Some(config) => {
                    let dir = config.get_temp_path().join(name);
                    ensure_cohesive_network_directory(network_descriptor, &dir).map_err(|e| {
                        CanisterStoreIdError::LoadCanisterIdStoreForNetwork {
                            network_descriptor_name: network_descriptor.name.clone(),
                            source: Box::new(e.into()),
                        }
                    })?;
                    Some(dir.join("canister_ids.json"))
                }
            },
        };
        let remote_ids = get_remote_ids(config);
        let ids = match &path {
            Some(path) if path.is_file() => dfx_core::json::load_json_file(path).map_err(|e| {
                CanisterStoreIdError::LoadCanisterIdStoreForNetwork {
                    network_descriptor_name: network_descriptor.name.clone(),
                    source: Box::new(e.into()),
                }
            })?,
            _ => CanisterIds::new(),
        };

        Ok(CanisterIdStore {
            network_descriptor: network_descriptor.clone(),
            path,
            ids,
            remote_ids,
        })
    }

    pub fn get_name(&self, canister_id: &str) -> Option<&String> {
        self.remote_ids
            .as_ref()
            .and_then(|remote_ids| self.get_name_in(canister_id, remote_ids))
            .or_else(|| self.get_name_in(canister_id, &self.ids))
    }

    pub fn get_name_in<'a, 'b>(
        &'a self,
        canister_id: &'b str,
        canister_ids: &'a CanisterIds,
    ) -> Option<&'a String> {
        canister_ids
            .iter()
            .find(|(_, nn)| nn.get(&self.network_descriptor.name) == Some(&canister_id.to_string()))
            .map(|(canister_name, _)| canister_name)
    }

    pub fn save_ids(&self) -> Result<(), CanisterStoreIdError> {
        let path = self
            .path
            .as_ref()
            .unwrap_or_else(|| {
                // the only callers of this method have already called Environment::get_config_or_anyhow
                unreachable!("Must be in a project (call Environment::get_config_or_anyhow()) to save canister ids")
            });
        let content = dfx_core::json::pretty_print(&self.ids)?;
        let parent = dfx_core::fs::parent(path)?;
        if !parent.exists() {
            dfx_core::fs::create_dir_all(&parent)?;
        }
        dfx_core::fs::write(path, content)?;
        Ok(())
    }

    pub fn find(&self, canister_name: &str) -> Option<CanisterId> {
        self.remote_ids
            .as_ref()
            .and_then(|remote_ids| self.find_in(canister_name, remote_ids))
            .or_else(|| self.find_in(canister_name, &self.ids))
    }

    fn find_in(&self, canister_name: &str, canister_ids: &CanisterIds) -> Option<CanisterId> {
        canister_ids
            .get(canister_name)
            .and_then(|network_name_to_canister_id| {
                network_name_to_canister_id
                    .get(&self.network_descriptor.name)
                    .or_else(|| network_name_to_canister_id.get(CanisterIdStore::DEFAULT))
            })
            .and_then(|s| CanisterId::from_text(s).ok())
    }

    pub fn get(&self, canister_name: &str) -> Result<CanisterId, CanisterStoreIdError> {
        self.find(canister_name).ok_or_else(|| {
            let network = if self.network_descriptor.name == "local" {
                "".to_string()
            } else {
                format!(" --network {}", self.network_descriptor.name)
            };
            CanisterStoreIdError::CanisterIdNotFound {
                canister_name: canister_name.to_string(),
                network,
            }
        })
    }

    pub fn add(
        &mut self,
        canister_name: &str,
        canister_id: &str,
    ) -> Result<(), CanisterStoreIdError> {
        let network_name = &self.network_descriptor.name;
        match self.ids.get_mut(canister_name) {
            Some(network_name_to_canister_id) => {
                network_name_to_canister_id
                    .insert(network_name.to_string(), canister_id.to_string());
            }
            None => {
                let mut network_name_to_canister_id = NetworkNametoCanisterId::new();
                network_name_to_canister_id
                    .insert(network_name.to_string(), canister_id.to_string());
                self.ids
                    .insert(canister_name.to_string(), network_name_to_canister_id);
            }
        }
        self.save_ids()
            .map_err(|e| CanisterStoreIdError::AddCanisterId {
                canister_name: canister_name.to_string(),
                canister_id: canister_id.to_string(),
                source: Box::new(e),
            })
    }

    pub fn remove(&mut self, canister_name: &str) -> Result<(), CanisterStoreIdError> {
        let network_name = &self.network_descriptor.name;
        if let Some(network_name_to_canister_id) = self.ids.get_mut(canister_name) {
            network_name_to_canister_id.remove(network_name);
        }
        self.save_ids()
            .map_err(|e| CanisterStoreIdError::RemoveCanisterId {
                canister_name: canister_name.to_string(),
                source: Box::new(e),
            })
    }
}

fn get_remote_ids(config: Option<Arc<Config>>) -> Option<CanisterIds> {
    let config = config?;
    let config = config.get_config();

    let mut remote_ids = CanisterIds::new();
    if let Some(canisters) = &config.canisters {
        for (canister_name, canister_config) in canisters {
            if let Some(remote) = &canister_config.remote {
                for (network_name, canister_id) in &remote.id {
                    let canister_id = canister_id.to_string();
                    match remote_ids.get_mut(canister_name) {
                        Some(network_name_to_canister_id) => {
                            network_name_to_canister_id
                                .insert(network_name.to_string(), canister_id);
                        }
                        None => {
                            let mut network_name_to_canister_id = NetworkNametoCanisterId::new();
                            network_name_to_canister_id
                                .insert(network_name.to_string(), canister_id);
                            remote_ids
                                .insert(canister_name.to_string(), network_name_to_canister_id);
                        }
                    }
                }
            }
        }
    }
    if remote_ids.is_empty() {
        None
    } else {
        Some(remote_ids)
    }
}
