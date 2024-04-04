use crate::config::model::dfinity::Config;
use crate::config::model::network_descriptor::{NetworkDescriptor, NetworkTypeDescriptor};
use crate::error::canister_id_store::CanisterIdStoreError;
use crate::error::unified_io::UnifiedIoError;
use crate::network::directory::ensure_cohesive_network_directory;
use candid::Principal as CanisterId;
use ic_agent::export::Principal;
use serde::{Deserialize, Serialize, Serializer};
use slog::{warn, Logger};
use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut, Sub};
use std::path::PathBuf;
use std::sync::Arc;
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

#[derive(Clone, Debug)]
pub struct CanisterIdStore {
    network_descriptor: NetworkDescriptor,
    canister_ids_path: Option<PathBuf>,
    canister_timestamps_path: Option<PathBuf>,

    // Only the canister ids read from/written to canister-ids.json
    // which does not include remote canister ids
    ids: CanisterIds,

    // Only canisters that will time out at some point have their timestamp of acquisition saved
    acquisition_timestamps: CanisterTimestamps,

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
                    ensure_cohesive_network_directory(network_descriptor, &dir).map_err(|e| {
                        CanisterIdStoreError::EnsureCohesiveNetworkDirectoryFailed {
                            network: network_descriptor.name.clone(),
                            cause: e.into(),
                        }
                    })?;
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
        let mut store = CanisterIdStore {
            network_descriptor: network_descriptor.clone(),
            canister_ids_path,
            canister_timestamps_path,
            ids,
            acquisition_timestamps,
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

    pub fn get_timestamp(&self, canister_name: &str) -> Option<&AcquisitionDateTime> {
        self.acquisition_timestamps
            .get(canister_name)
            .and_then(|timestamp_map| timestamp_map.get(&self.network_descriptor.name))
    }

    pub fn get_name(&self, canister_id: &str) -> Option<&String> {
        self.remote_ids
            .as_ref()
            .and_then(|remote_ids| self.get_name_in(canister_id, remote_ids))
            .or_else(|| self.get_name_in_project(canister_id))
            .or_else(|| self.get_name_in_pull_ids(canister_id))
    }

    pub fn get_name_in_project(&self, canister_id: &str) -> Option<&String> {
        self.get_name_in(canister_id, &self.ids)
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

    pub fn save_ids(&self) -> Result<(), UnifiedIoError> {
        let path = self
            .canister_ids_path
            .as_ref()
            .unwrap_or_else(|| {
                // the only callers of this method have already called Environment::get_config_or_anyhow
                unreachable!("Must be in a project (call Environment::get_config_or_anyhow()) to save canister ids")
            });
        crate::fs::composite::ensure_parent_dir_exists(path)?;
        crate::json::save_json_file(path, &self.ids)?;
        Ok(())
    }

    fn save_timestamps(&self) -> Result<(), CanisterIdStoreError> {
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
            .or_else(|| self.find_in(canister_name, &self.ids))
            .or_else(|| self.pull_ids.get(canister_name).copied())
    }
    pub fn get_name_id_map(&self) -> BTreeMap<String, String> {
        let mut ids: BTreeMap<_, _> = self
            .ids
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
        &mut self,
        canister_name: &str,
        canister_id: &str,
        timestamp: Option<AcquisitionDateTime>,
    ) -> Result<(), CanisterIdStoreError> {
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
            .map_err(|e| CanisterIdStoreError::AddCanisterId {
                canister_name: canister_name.to_string(),
                canister_id: canister_id.to_string(),
                cause: e,
            })?;
        if let Some(timestamp) = timestamp {
            match self.acquisition_timestamps.get_mut(canister_name) {
                Some(network_name_to_timestamp) => {
                    network_name_to_timestamp.insert(network_name.to_string(), timestamp);
                }
                None => {
                    let mut network_name_to_timestamp = NetworkNametoCanisterTimestamp::default();
                    network_name_to_timestamp.insert(network_name.to_string(), timestamp);
                    self.acquisition_timestamps
                        .insert(canister_name.to_string(), network_name_to_timestamp);
                }
            }
            self.save_timestamps()?;
        }
        Ok(())
    }

    pub fn remove(&mut self, canister_name: &str) -> Result<(), CanisterIdStoreError> {
        let network_name = &self.network_descriptor.name;
        if let Some(network_name_to_canister_id) = self.ids.get_mut(canister_name) {
            network_name_to_canister_id.remove(network_name);
            self.save_ids()
                .map_err(|e| CanisterIdStoreError::RemoveCanisterId {
                    canister_name: canister_name.to_string(),
                    cause: e,
                })?
        };
        if let Some(network_name_to_timestamp) = self.acquisition_timestamps.get_mut(canister_name)
        {
            network_name_to_timestamp.remove(network_name);
            self.save_timestamps()?;
        }
        Ok(())
    }

    fn prune_expired_canisters(
        &mut self,
        log: &Logger,
        timeout: &Duration,
    ) -> Result<(), CanisterIdStoreError> {
        let network_name = &self.network_descriptor.name;
        let now = SystemTime::now();
        let prune_cutoff = now.sub(*timeout);

        let mut canisters_to_prune: Vec<String> = Vec::new();
        for (canister_name, timestamp) in self.acquisition_timestamps.iter().filter_map(
            |(canister_name, network_to_timestamp)| {
                network_to_timestamp
                    .get(network_name)
                    .map(|timestamp| (canister_name, timestamp))
            },
        ) {
            if *timestamp <= prune_cutoff {
                canisters_to_prune.push(canister_name.clone());
            }
        }

        for canister in canisters_to_prune {
            warn!(log, "Canister '{}' has timed out.", &canister);
            self.remove(&canister)?;
        }

        Ok(())
    }

    pub fn non_remote_user_canisters(&self) -> Vec<(String, Principal)> {
        self.ids
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
