#![allow(dead_code)]
use crate::config::dfinity::Config;
use crate::lib::canister_info::assets::AssetsCanisterInfo;
use crate::lib::canister_info::motoko::MotokoCanisterInfo;
use crate::lib::error::{DfxError, DfxResult};
use ic_agent::{Blob, CanisterId};
use rand::{thread_rng, RngCore};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub mod motoko;
pub mod assets;

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

    canister_id: RefCell<Option<CanisterId>>,
    canister_id_path: PathBuf,

    packtool: Option<String>,

    extras: BTreeMap<String, serde_json::Value>,
}

impl CanisterInfo {
    pub fn load(config: &Config, name: &str) -> DfxResult<CanisterInfo> {
        let workspace_root = config.get_path().parent().unwrap();
        let build_defaults = config.get_config().get_defaults().get_build();
        let build_root = workspace_root.join(build_defaults.get_output("build/"));

        let canister_map = (&config.get_config().canisters).as_ref().ok_or_else(|| {
            DfxError::Unknown("No canisters in the configuration file.".to_string())
        })?;

        let canister_config = canister_map
            .get(name)
            .ok_or_else(|| DfxError::CannotFindCanisterName(name.to_string()))?;

        let canister_root = workspace_root.to_path_buf();
        let extras = canister_config.extras.clone();

        let output_root = build_root.join(name);
        let canister_id_path = output_root.join("_canister.id");
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

            canister_id: RefCell::new(None),
            canister_id_path,

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
    pub fn get_canister_id_path(&self) -> &Path {
        self.canister_id_path.as_path()
    }

    pub fn get_canister_id(&self) -> Option<CanisterId> {
        let canister_id = self.canister_id.replace(None).or_else(|| {
            std::fs::read(&self.canister_id_path)
                .ok()
                .map(|cid| CanisterId::from(Blob::from(cid)))
        });

        self.canister_id.replace(canister_id.clone());

        canister_id
    }

    pub fn get_extra_value(&self, name: &str) -> Option<serde_json::Value> {
        self.extras.get(name).cloned()
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

    pub fn get_output_wasm_path(&self) -> Option<PathBuf> {
        if let Ok(info) = self.as_info::<MotokoCanisterInfo>() {
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
        } else if let Ok(info) = self.as_info::<AssetsCanisterInfo>() {
            Some(info.get_output_idl_path().to_path_buf())
        } else {
            None
        }
    }

    pub fn generate_canister_id(&self) -> DfxResult<CanisterId> {
        let mut rng = thread_rng();
        let mut v: Vec<u8> = std::iter::repeat(0u8).take(8).collect();
        rng.fill_bytes(v.as_mut_slice());

        Ok(CanisterId::from(Blob(v)))
    }

    pub fn as_info<T: CanisterInfoFactory>(&self) -> DfxResult<T> {
        if T::supports(self) {
            T::create(self)
        } else {
            Err(DfxError::InvalidCanisterType(self.get_type().to_string()))
        }
    }
}
