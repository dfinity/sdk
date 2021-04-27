use crate::lib::canister_info::{CanisterInfo, CanisterInfoFactory};
use crate::lib::error::DfxResult;
use std::path::{Path, PathBuf};

pub struct CustomCanisterInfo {
    output_wasm_path: PathBuf,
    output_idl_path: PathBuf,
    output_assets_path: Option<PathBuf>,
}

impl CustomCanisterInfo {
    pub fn get_output_wasm_path(&self) -> &Path {
        self.output_wasm_path.as_path()
    }
    pub fn get_output_idl_path(&self) -> &Path {
        self.output_idl_path.as_path()
    }
    pub fn get_output_assets_path(&self) -> Option<&Path> {
        self.output_assets_path.as_ref().map(|p| p.as_path())
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
        let output_assets_path =
            if info.has_extra("assets") {
                Some(workspace_root.join(info.get_extra::<PathBuf>("assets")?))
            } else {
                None
            };

        Ok(Self {
            output_wasm_path,
            output_idl_path,
            output_assets_path
        })
    }
}
