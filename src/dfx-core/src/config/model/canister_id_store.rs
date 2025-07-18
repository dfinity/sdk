use crate::config::model::dfinity::Config;
use crate::config::model::network_descriptor::{NetworkDescriptor, NetworkTypeDescriptor};
use crate::error::canister_id_store::{
    AddCanisterIdError, CanisterIdStoreError, RemoveCanisterIdError, SaveIdsError,
    SaveTimestampsError,
};
use crate::network::directory::ensure_cohesive_network_directory;
use candid::Principal as CanisterId;
use ic_agent::export::Principal;
use serde::{Deserialize, Serialize, Serializer};
use slog::{warn, Logger};
use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut, Sub};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

pub type CanisterName = String;
pub type NetworkName = String;
pub type CanisterIdString = String;

pub type NetworkNametoCanisterId = BTreeMap<NetworkName, CanisterIdString>;
pub type CanisterIds = BTreeMap<CanisterName, NetworkNametoCanisterId>;

pub type CanisterTimestamps = BTreeMap<CanisterName, NetworkNametoCanisterTimestamp>;

// OffsetDateTime has nanosecond precision, while SystemTime is OS-dependent (100ns on Windows)
pub type AcquisitionDateTime = OffsetDateTime;

#[derive(Debug, Clone, Default)]
pub struct NetworkNametoCanisterTimestamp(BTreeMap<NetworkName, AcquisitionDateTime>);

impl Deref for NetworkNametoCanisterTimestamp {
    type Target = BTreeMap<NetworkName, AcquisitionDateTime>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for NetworkNametoCanisterTimestamp {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Serialize for NetworkNametoCanisterTimestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let out = self.0.iter().map(|(key, time)| {
            (
                key,
                AcquisitionDateTime::from(*time)
                    .format(&Rfc3339)
                    .expect("Failed to serialise timestamp"),
            )
        });
        serializer.collect_map(out)
    }
}

impl<'de> Deserialize<'de> for NetworkNametoCanisterTimestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let map: BTreeMap<NetworkName, String> = Deserialize::deserialize(deserializer)?;
        let btree: BTreeMap<NetworkName, AcquisitionDateTime> = map
            .into_iter()
            .map(|(key, timestamp)| (key, AcquisitionDateTime::parse(&timestamp, &Rfc3339)))
            .try_fold(BTreeMap::new(), |mut map, (key, result)| match result {
                Ok(value) => {
                    map.insert(key, value);
                    Ok(map)
                }
                Err(err) => Err(err),
            })
            .map_err(|err| serde::de::Error::custom(err.to_string()))?;
        Ok(Self(btree))
    }
}

#[derive(Debug)]
pub struct CanisterIdStore {
    network_descriptor: NetworkDescriptor,
    canister_ids_path: Option<PathBuf>,
    canister_timestamps_path: Option<PathBuf>,

    // Only the canister ids read from/written to canister-ids.json
    // which does not include remote canister ids
    ids: Mutex<CanisterIds>,

    // Only canisters that will time out at some point have their timestamp of acquisition saved
    acquisition_timestamps: Mutex<CanisterTimestamps>,

    // Remote ids read from dfx.json, never written to canister_ids.json
    remote_ids: Option<CanisterIds>,

    // ids of pull dependencies in dfx.json, never written to canister_ids.json
    pull_ids: BTreeMap<CanisterName, CanisterId>,
}

impl CanisterIdStore {
    pub const DEFAULT: &'static str = "__default";

    pub fn new(
        log: &Logger,
        network_descriptor: &NetworkDescriptor,
        config: Option<Arc<Config>>,
    ) -> Result<Self, CanisterIdStoreError> {
        let canister_ids_path = match network_descriptor {
            NetworkDescriptor {
                r#type: NetworkTypeDescriptor::Persistent,
                ..
            } => config
                .as_ref()
                .map(|c| c.get_project_root().join("canister_ids.json")),
            NetworkDescriptor { name, .. } => match &config {
                None => None,
                Some(config) => {
                    let dir = config.get_temp_path()?.join(name);
                    ensure_cohesive_network_directory(network_descriptor, &dir)?;
                    Some(dir.join("canister_ids.json"))
                }
            },
        };
        let canister_timestamps_path = match network_descriptor {
            NetworkDescriptor {
                name,
                r#type: NetworkTypeDescriptor::Playground { .. },
                ..
            } => {
                if let Some(config) = config.as_ref() {
                    let dir = config.get_temp_path()?.join(name);
                    ensure_cohesive_network_directory(network_descriptor, &dir)?;
                    Some(dir.join("canister_timestamps.json"))
                } else {
                    None
                }
            }
            _ => None,
        };
        let remote_ids = get_remote_ids(config.clone());
        let pull_ids = if let Some(config) = config {
            config.get_config().get_pull_canisters()?
        } else {
            BTreeMap::new()
        };
        let ids = match &canister_ids_path {
            Some(path) if path.is_file() => crate::json::load_json_file(path)?,
            _ => CanisterIds::new(),
        };
        let acquisition_timestamps = match &canister_timestamps_path {
            Some(path) if path.is_file() => crate::json::load_json_file(path)?,
            _ => CanisterTimestamps::new(),
        };
        let store = CanisterIdStore {
            network_descriptor: network_descriptor.clone(),
            canister_ids_path,
            canister_timestamps_path,
            ids: Mutex::new(ids),
            acquisition_timestamps: Mutex::new(acquisition_timestamps),
            remote_ids,
            pull_ids,
        };

        if let NetworkTypeDescriptor::Playground {
            canister_timeout_seconds,
            ..
        } = &network_descriptor.r#type
        {
            store.prune_expired_canisters(log, &Duration::from_secs(*canister_timeout_seconds))?;
        }

        Ok(store)
    }

    pub fn get_timestamp(&self, canister_name: &str) -> Option<AcquisitionDateTime> {
        self.acquisition_timestamps
            .lock()
            .unwrap()
            .get(canister_name)
            .and_then(|timestamp_map| timestamp_map.get(&self.network_descriptor.name).copied())
    }

    pub fn get_name(&self, canister_id: &str) -> Option<String> {
        self.remote_ids
            .as_ref()
            .and_then(|remote_ids| self.get_name_in(canister_id, remote_ids).cloned())
            .or_else(|| self.get_name_in_project(canister_id))
            .or_else(|| self.get_name_in_pull_ids(canister_id).cloned())
    }

    pub fn get_name_in_project(&self, canister_id: &str) -> Option<String> {
        self.get_name_in(canister_id, &self.ids.lock().unwrap())
            .cloned()
    }

    pub fn get_name_in<'a>(
        &'a self,
        canister_id: &str,
        canister_ids: &'a CanisterIds,
    ) -> Option<&'a String> {
        canister_ids
            .iter()
            .find(|(_, nn)| nn.get(&self.network_descriptor.name) == Some(&canister_id.to_string()))
            .map(|(canister_name, _)| canister_name)
    }

    fn get_name_in_pull_ids(&self, canister_id: &str) -> Option<&String> {
        self.pull_ids
            .iter()
            .find(|(_, id)| id.to_text() == canister_id)
            .map(|(canister_name, _)| canister_name)
    }

    fn warn_for_canister_ids_path(&self) -> bool {
        // Only warn when the 'canister_ids.json' file is first generated under the project root directory for persistent networks.
        if let NetworkDescriptor {
            r#type: NetworkTypeDescriptor::Persistent,
            ..
        } = self.network_descriptor
        {
            if let Some(path) = self.canister_ids_path.as_ref() {
                if !path.exists() {
                    return true;
                }
            }
        };

        false
    }

    pub fn save_ids(&self, log: &Logger) -> Result<(), SaveIdsError> {
        let path = self
            .canister_ids_path
            .as_ref()
            .unwrap_or_else(|| {
                // the only callers of this method have already called Environment::get_config_or_anyhow
                unreachable!("Must be in a project (call Environment::get_config_or_anyhow()) to save canister ids")
            });
        let to_warn = self.warn_for_canister_ids_path();
        crate::fs::composite::ensure_parent_dir_exists(path)?;
        crate::json::save_json_file(path, &self.ids)?;
        if to_warn {
            warn!(log, "The {:?} file has been generated. Please make sure you store it correctly, e.g., submitting it to a GitHub repository.", path);
        }
        Ok(())
    }

    fn save_timestamps(&self) -> Result<(), SaveTimestampsError> {
        let path = self
            .canister_timestamps_path
            .as_ref()
            .unwrap_or_else(|| {
                // the only callers of this method have already called Environment::get_config_or_anyhow
                unreachable!("Must be in a project (call Environment::get_config_or_anyhow()) to save canister timestamps")
            });
        crate::fs::composite::ensure_parent_dir_exists(path)?;
        crate::json::save_json_file(path, &self.acquisition_timestamps)?;
        Ok(())
    }

    pub fn find(&self, canister_name: &str) -> Option<CanisterId> {
        self.remote_ids
            .as_ref()
            .and_then(|remote_ids| self.find_in(canister_name, remote_ids))
            .or_else(|| self.find_in(canister_name, &self.ids.lock().unwrap()))
            .or_else(|| self.pull_ids.get(canister_name).copied())
    }
    pub fn get_name_id_map(&self) -> BTreeMap<String, String> {
        let mut ids: BTreeMap<_, _> = self
            .ids
            .lock()
            .unwrap()
            .iter()
            .filter_map(|(name, network_to_id)| {
                Some((
                    name.clone(),
                    network_to_id.get(&self.network_descriptor.name).cloned()?,
                ))
            })
            .collect();
        if let Some(remote_ids) = &self.remote_ids {
            let mut remote = remote_ids
                .iter()
                .filter_map(|(name, network_to_id)| {
                    Some((
                        name.clone(),
                        network_to_id.get(&self.network_descriptor.name).cloned()?,
                    ))
                })
                .collect();
            ids.append(&mut remote);
        }
        let mut pull_ids = self
            .pull_ids
            .iter()
            .map(|(name, id)| (name.clone(), id.to_text()))
            .collect();
        ids.append(&mut pull_ids);
        ids.into_iter()
            .filter(|(name, _)| !name.starts_with("__"))
            .collect()
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

    pub fn get(&self, canister_name: &str) -> Result<CanisterId, CanisterIdStoreError> {
        self.find(canister_name).ok_or_else(|| {
            let network = if self.network_descriptor.name == "local" {
                "".to_string()
            } else {
                format!(" --network {}", self.network_descriptor.name)
            };
            CanisterIdStoreError::CanisterIdNotFound {
                canister_name: canister_name.to_string(),
                network,
            }
        })
    }

    pub fn add(
        &self,
        log: &Logger,
        canister_name: &str,
        canister_id: &str,
        timestamp: Option<AcquisitionDateTime>,
    ) -> Result<(), AddCanisterIdError> {
        let network_name = &self.network_descriptor.name;
        self.ids
            .lock()
            .unwrap()
            .entry(canister_name.to_string())
            .or_default()
            .insert(network_name.to_string(), canister_id.to_string());
        self.save_ids(log)
            .map_err(|source| AddCanisterIdError::SaveIds {
                canister_name: canister_name.to_string(),
                canister_id: canister_id.to_string(),
                source,
            })?;
        if let Some(timestamp) = timestamp {
            self.acquisition_timestamps
                .lock()
                .unwrap()
                .entry(canister_name.to_string())
                .or_default()
                .insert(network_name.to_string(), timestamp);

            self.save_timestamps()?;
        }
        Ok(())
    }

    pub fn remove(&self, log: &Logger, canister_name: &str) -> Result<(), RemoveCanisterIdError> {
        let network_name = &self.network_descriptor.name;
        let save = if let Some(network_name_to_canister_id) =
            self.ids.lock().unwrap().get_mut(canister_name)
        {
            network_name_to_canister_id.remove(network_name);
            true
        } else {
            false
        };
        if save {
            self.save_ids(log)
                .map_err(|e| RemoveCanisterIdError::SaveIds {
                    canister_name: canister_name.to_string(),
                    source: e,
                })?
        }
        let save = if let Some(network_name_to_timestamp) = self
            .acquisition_timestamps
            .lock()
            .unwrap()
            .get_mut(canister_name)
        {
            network_name_to_timestamp.remove(network_name);
            true
        } else {
            false
        };
        if save {
            self.save_timestamps()?;
        }
        Ok(())
    }

    fn prune_expired_canisters(
        &self,
        log: &Logger,
        timeout: &Duration,
    ) -> Result<(), RemoveCanisterIdError> {
        let network_name = &self.network_descriptor.name;
        let now = SystemTime::now();
        let prune_cutoff = now.sub(*timeout);

        let mut canisters_to_prune: Vec<String> = Vec::new();
        for (canister_name, timestamp) in self
            .acquisition_timestamps
            .lock()
            .unwrap()
            .iter()
            .filter_map(|(canister_name, network_to_timestamp)| {
                network_to_timestamp
                    .get(network_name)
                    .map(|timestamp| (canister_name, timestamp))
            })
        {
            if *timestamp <= prune_cutoff {
                canisters_to_prune.push(canister_name.clone());
            }
        }

        for canister in canisters_to_prune {
            warn!(log, "Canister '{}' has timed out.", &canister);
            self.remove(log, &canister)?;
        }

        Ok(())
    }

    pub fn non_remote_user_canisters(&self) -> Vec<(String, Principal)> {
        self.ids
            .lock()
            .unwrap()
            .iter()
            .filter_map(|(name, network_to_id)| {
                network_to_id
                    .get(&self.network_descriptor.name)
                    .and_then(|principal| Principal::from_text(principal).ok())
                    .map(|principal| (name.clone(), principal))
            })
            .collect()
    }
}

impl Clone for CanisterIdStore {
    // Mutex doesn't impl Clone
    fn clone(&self) -> Self {
        CanisterIdStore {
            network_descriptor: self.network_descriptor.clone(),
            canister_ids_path: self.canister_ids_path.clone(),
            canister_timestamps_path: self.canister_timestamps_path.clone(),
            ids: Mutex::new(self.ids.lock().unwrap().clone()),
            acquisition_timestamps: Mutex::new(self.acquisition_timestamps.lock().unwrap().clone()),
            remote_ids: self.remote_ids.clone(),
            pull_ids: self.pull_ids.clone(),
        }
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
