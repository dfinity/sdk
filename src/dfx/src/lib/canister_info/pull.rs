use crate::lib::canister_info::{CanisterInfo, CanisterInfoFactory};
use crate::lib::deps::{get_candid_path_in_project, get_pulled_wasm_path};
use crate::lib::error::DfxResult;

use std::path::{Path, PathBuf};

use anyhow::bail;
use candid::Principal;
use dfx_core::config::model::dfinity::CanisterTypeProperties;

pub struct PullCanisterInfo {
    name: String,
    canister_id: Principal,
    output_wasm_path: PathBuf,
    output_idl_path: PathBuf,
}

impl PullCanisterInfo {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_canister_id(&self) -> &Principal {
        &self.canister_id
    }

    pub fn get_output_wasm_path(&self) -> &Path {
        self.output_wasm_path.as_path()
    }

    pub fn get_output_idl_path(&self) -> &Path {
        self.output_idl_path.as_path()
    }
}

impl CanisterInfoFactory for PullCanisterInfo {
    fn create(info: &CanisterInfo) -> DfxResult<Self> {
        let name = info.get_name().to_string();
        let canister_id = {
            if let CanisterTypeProperties::Pull { id } = info.type_specific.clone() {
                id
            } else {
                bail!(
                    "Attempted to construct a pull canister from a type:{} canister config",
                    info.type_specific.name()
                );
            }
        };

        let output_wasm_path = get_pulled_wasm_path(canister_id)?;

        let workspace_root = info.get_workspace_root().to_path_buf();
        let output_idl_path = get_candid_path_in_project(&workspace_root, &name);

        Ok(Self {
            name,
            canister_id,
            output_wasm_path,
            output_idl_path,
        })
    }
}
