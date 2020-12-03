use crate::config::dfinity::NetworkType;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::network::network_descriptor::NetworkDescriptor;

use anyhow::{anyhow, Context};
use ic_types::principal::Principal as CanisterId;
use std::collections::BTreeMap;
use std::path::PathBuf;

type CanisterName = String;
type NetworkName = String;
type CanisterIdString = String;

type NetworkNametoCanisterId = BTreeMap<NetworkName, CanisterIdString>;
type CanisterIds = BTreeMap<CanisterName, NetworkNametoCanisterId>;

#[derive(Clone, Debug)]
pub struct CanisterIdStore {
    pub network_descriptor: NetworkDescriptor,
    pub path: PathBuf,
    pub ids: CanisterIds,
}

impl CanisterIdStore {
    pub fn for_env(env: &dyn Environment) -> DfxResult<Self> {
        let network_descriptor = env.get_network_descriptor().expect("no network descriptor");
        CanisterIdStore::for_network(network_descriptor)
    }

    pub fn for_network(network_descriptor: &NetworkDescriptor) -> DfxResult<Self> {
        let path = match network_descriptor {
            NetworkDescriptor {
                r#type: NetworkType::Persistent,
                ..
            } => PathBuf::from("canister_ids.json"),
            NetworkDescriptor { name, .. } => {
                PathBuf::from(&format!(".dfx/{}/canister_ids.json", name))
            }
        };
        let ids = if path.is_file() {
            CanisterIdStore::load_ids(&path)?
        } else {
            CanisterIds::new()
        };

        Ok(CanisterIdStore {
            network_descriptor: network_descriptor.clone(),
            path,
            ids,
        })
    }

    pub fn get_name(&self, canister_id: &str) -> Option<&String> {
        self.ids
            .iter()
            .find(|(_, nn)| nn.get(&self.network_descriptor.name) == Some(&canister_id.to_string()))
            .map(|(canister_name, _)| canister_name)
    }

    pub fn load_ids(path: &PathBuf) -> DfxResult<CanisterIds> {
        let content = std::fs::read_to_string(path)
            .context(format!("Cannot read from file at '{}'.", path.display()))?;
        serde_json::from_str(&content).context(format!(
            "Cannot decode contents of file at '{}'.",
            path.display()
        ))
    }

    pub fn save_ids(&self) -> DfxResult {
        let content = serde_json::to_string_pretty(&self.ids)?;
        let parent = self.path.parent().unwrap();
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.path, content).context(format!(
            "Cannot write to file at '{}'.",
            self.path.display()
        ))
    }

    pub fn find(&self, canister_name: &str) -> Option<CanisterId> {
        self.ids
            .get(canister_name)
            .and_then(|network_name_to_canister_id| {
                network_name_to_canister_id.get(&self.network_descriptor.name)
            })
            .and_then(|s| CanisterId::from_text(s).ok())
    }

    pub fn get(&self, canister_name: &str) -> DfxResult<CanisterId> {
        self.find(canister_name).ok_or_else(|| {
            let network = if self.network_descriptor.name == "local" {
                "".to_string()
            } else {
                format!("--network {} ", self.network_descriptor.name)
            };
            anyhow!(
                "Cannot find canister id. Please issue 'dfx canister {}create {}'.",
                network,
                canister_name,
            )
        })
    }

    pub fn add(&mut self, canister_name: &str, canister_id: String) -> DfxResult<()> {
        let network_name = &self.network_descriptor.name;
        match self.ids.get_mut(canister_name) {
            Some(network_name_to_canister_id) => {
                network_name_to_canister_id.insert(network_name.to_string(), canister_id);
            }
            None => {
                let mut network_name_to_canister_id = NetworkNametoCanisterId::new();
                network_name_to_canister_id.insert(network_name.to_string(), canister_id);
                self.ids
                    .insert(canister_name.to_string(), network_name_to_canister_id);
            }
        }
        self.save_ids()
    }

    pub fn remove(&mut self, canister_name: &str) -> DfxResult<()> {
        let network_name = &self.network_descriptor.name;
        if let Some(network_name_to_canister_id) = self.ids.get_mut(canister_name) {
            network_name_to_canister_id.remove(&network_name.to_string());
            self.ids.remove(&canister_name.to_string());
        }
        self.save_ids()
    }
}
