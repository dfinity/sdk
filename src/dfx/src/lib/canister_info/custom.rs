use anyhow::bail;

use crate::config::dfinity::CanisterTypeProperties;
use crate::lib::canister_info::{CanisterInfo, CanisterInfoFactory};
use crate::lib::error::DfxResult;
use std::path::{Path, PathBuf};

pub struct CustomCanisterInfo {
    output_wasm_path: PathBuf,
    output_idl_path: PathBuf,
    build: Vec<String>,
}

impl CustomCanisterInfo {
    pub fn get_output_wasm_path(&self) -> &Path {
        self.output_wasm_path.as_path()
    }
    pub fn get_output_idl_path(&self) -> &Path {
        self.output_idl_path.as_path()
    }
    pub fn get_build_tasks(&self) -> &[String] {
        &self.build
    }
}

impl CanisterInfoFactory for CustomCanisterInfo {
    fn create(info: &CanisterInfo) -> DfxResult<Self> {
        let workspace_root = info.get_workspace_root();
        let (wasm, build, candid) = if let CanisterTypeProperties::Custom {
            wasm,
            build,
            candid,
        } = info.type_specific.clone()
        {
            (wasm, build.into_vec(), candid)
        } else {
            bail!(
                "Attempted to construct a custom canister from a type:{} canister config",
                info.type_specific.name()
            )
        };
        let output_wasm_path = workspace_root.join(wasm);
        let candid = if let Some(remote_candid) = info.get_remote_candid_if_remote() {
            remote_candid
        } else {
            candid
        };
        let output_idl_path = workspace_root.join(candid);

        Ok(Self {
            output_wasm_path,
            output_idl_path,
            build,
        })
    }
}
