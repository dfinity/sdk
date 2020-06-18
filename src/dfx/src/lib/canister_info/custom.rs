use crate::lib::canister_info::{CanisterInfo, CanisterInfoFactory};
use crate::lib::error::DfxResult;
use std::path::{Path, PathBuf};

pub struct CustomCanisterInfo {
    output_wasm_path: PathBuf,
    output_idl_path: PathBuf,
}

impl CustomCanisterInfo {
    pub fn get_output_wasm_path(&self) -> &Path {
        self.output_wasm_path.as_path()
    }
    pub fn get_output_idl_path(&self) -> &Path {
        self.output_idl_path.as_path()
    }
}

impl CanisterInfoFactory for CustomCanisterInfo {
    fn supports(info: &CanisterInfo) -> bool {
        info.get_type() == "custom"
    }

    fn create(info: &CanisterInfo) -> DfxResult<Self> {
        let workspace_root = info.get_workspace_root();
        let output_wasm_path = workspace_root.join(info.get_extra::<PathBuf>("wasm")?);
        let output_idl_path = workspace_root.join(info.get_extra::<PathBuf>("candid")?);

        Ok(Self {
            output_wasm_path,
            output_idl_path,
        })
    }
}
