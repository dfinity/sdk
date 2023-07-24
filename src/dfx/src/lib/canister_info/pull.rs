use crate::lib::canister_info::{CanisterInfo, CanisterInfoFactory};
use crate::lib::deps::get_candid_path_in_project;
use crate::lib::error::DfxResult;
use anyhow::bail;
use candid::Principal;
use dfx_core::config::model::dfinity::CanisterTypeProperties;
use std::path::{Path, PathBuf};

pub struct PullCanisterInfo {
    name: String,
    canister_id: Principal,
    output_idl_path: PathBuf,
}

impl PullCanisterInfo {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_canister_id(&self) -> &Principal {
        &self.canister_id
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

        let workspace_root = info.get_workspace_root().to_path_buf();
        let output_idl_path = get_candid_path_in_project(&workspace_root, &canister_id);

        Ok(Self {
            name,
            canister_id,
            output_idl_path,
        })
    }
}
