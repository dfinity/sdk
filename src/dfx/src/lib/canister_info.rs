#![allow(dead_code)]
use crate::config::dfinity::Config;
use crate::lib::canister_info::assets::AssetsCanisterInfo;
use crate::lib::canister_info::custom::CustomCanisterInfo;
use crate::lib::canister_info::motoko::MotokoCanisterInfo;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::provider::get_network_context;
use ic_agent::CanisterId;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub mod assets;
pub mod custom;
pub mod motoko;

pub trait CanisterInfoFactory {
    /// Returns true if this factory supports creating extra info for this canister info.
    fn supports(info: &CanisterInfo) -> bool;

    fn create(info: &CanisterInfo) -> DfxResult<Self>
    where
        Self: std::marker::Sized;
}

/// Information about a canister project (source code, destination, etc).
#[derive(Debug)]
pub struct CanisterInfo {
    name: String,
    canister_type: String,

    workspace_root: PathBuf,
    build_root: PathBuf,
    output_root: PathBuf,
    canister_root: PathBuf,

    canister_id: Option<CanisterId>,

    packtool: Option<String>,

    extras: BTreeMap<String, serde_json::Value>,
}

impl CanisterInfo {
    pub fn load(
        config: &Config,
        name: &str,
        canister_id: Option<CanisterId>,
    ) -> DfxResult<CanisterInfo> {
        let workspace_root = config.get_path().parent().unwrap();
        let build_defaults = config.get_config().get_defaults().get_build();
        let network_name = get_network_context().ok_or_else(|| DfxError::ComputeNetworkNotSet)?;
        let build_root = config.get_temp_path().join(network_name);
        let build_root = build_root.join(build_defaults.get_output());
        std::fs::create_dir_all(&build_root)?;

        let canister_map = (&config.get_config().canisters).as_ref().ok_or_else(|| {
            DfxError::Unknown("No canisters in the configuration file.".to_string())
        })?;

        let canister_config = canister_map
            .get(name)
            .ok_or_else(|| DfxError::CannotFindCanisterName(name.to_string()))?;

        let canister_root = workspace_root.to_path_buf();
        let extras = canister_config.extras.clone();

        let output_root = build_root.join(name);

        let canister_type = canister_config
            .r#type
            .as_ref()
            .cloned()
            .unwrap_or_else(|| "motoko".to_owned());

        Ok(CanisterInfo {
            name: name.to_string(),
            canister_type,

            workspace_root: workspace_root.to_path_buf(),
            build_root,
            output_root,
            canister_root,

            canister_id,

            packtool: build_defaults.get_packtool(),
            extras,
        })
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }
    pub fn get_type(&self) -> &str {
        &self.canister_type
    }
    pub fn get_workspace_root(&self) -> &Path {
        &self.workspace_root
    }
    pub fn get_build_root(&self) -> &Path {
        &self.build_root
    }
    pub fn get_output_root(&self) -> &Path {
        &self.output_root
    }
    pub fn get_canister_id(&self) -> DfxResult<CanisterId> {
        match &self.canister_id {
            Some(canister_id) => Ok(canister_id.clone()),
            None => {
                // If we get here, it means there is a logic error in the code.
                // It's not because the user did anything in the wrong order.
                // We need the network type (ephemeral/persistent) in order to load
                // the canister id, so we can't load it here.
                panic!("It is only valid to call get_canister_id after setting the canister id.");
            }
        }
    }

    pub fn get_extra_value(&self, name: &str) -> Option<serde_json::Value> {
        self.extras.get(name).cloned()
    }

    pub fn has_extra(&self, name: &str) -> bool {
        self.extras.contains_key(name)
    }

    pub fn get_extra<T: serde::de::DeserializeOwned>(&self, name: &str) -> DfxResult<T> {
        self.get_extra_value(name)
            .ok_or_else(|| {
                DfxError::Unknown(format!(
                    "Field '{}' is mandatory for canister {}.",
                    name,
                    self.get_name()
                ))
            })
            .and_then(|v| {
                T::deserialize(v).map_err(|_| {
                    DfxError::Unknown(format!("Field '{}' is of the wrong type", name))
                })
            })
    }
    pub fn get_extras(&self) -> &BTreeMap<String, serde_json::Value> {
        &self.extras
    }

    pub fn get_packtool(&self) -> &Option<String> {
        &self.packtool
    }

    pub fn get_build_wasm_path(&self) -> PathBuf {
        self.build_root
            .join(PathBuf::from(&self.name))
            .join(&self.name)
            .with_extension("wasm")
            .to_path_buf()
    }

    pub fn get_build_idl_path(&self) -> PathBuf {
        self.build_root
            .join(PathBuf::from(&self.name))
            .join(&self.name)
            .with_extension("did")
            .to_path_buf()
    }

    pub fn get_output_wasm_path(&self) -> Option<PathBuf> {
        if let Ok(info) = self.as_info::<MotokoCanisterInfo>() {
            Some(info.get_output_wasm_path().to_path_buf())
        } else if let Ok(info) = self.as_info::<CustomCanisterInfo>() {
            Some(info.get_output_wasm_path().to_path_buf())
        } else if let Ok(info) = self.as_info::<AssetsCanisterInfo>() {
            Some(info.get_output_wasm_path().to_path_buf())
        } else {
            None
        }
    }

    pub fn get_output_idl_path(&self) -> Option<PathBuf> {
        if let Ok(info) = self.as_info::<MotokoCanisterInfo>() {
            Some(info.get_output_idl_path().to_path_buf())
        } else if let Ok(info) = self.as_info::<CustomCanisterInfo>() {
            Some(info.get_output_idl_path().to_path_buf())
        } else if let Ok(info) = self.as_info::<AssetsCanisterInfo>() {
            Some(info.get_output_idl_path().to_path_buf())
        } else {
            None
        }
    }

    pub fn as_info<T: CanisterInfoFactory>(&self) -> DfxResult<T> {
        if T::supports(self) {
            T::create(self)
        } else {
            Err(DfxError::InvalidCanisterType(self.get_type().to_string()))
        }
    }
}
