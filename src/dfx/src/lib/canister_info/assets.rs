use crate::lib::canister_info::{CanisterInfo, CanisterInfoFactory};
use crate::lib::error::DfxResult;
use std::path::{Path, PathBuf};

pub struct AssetsCanisterInfo {
    output_wasm_path: PathBuf,
    output_idl_path: PathBuf,
}

impl AssetsCanisterInfo {
    pub fn get_output_wasm_path(&self) -> &Path {
        self.output_wasm_path.as_path()
    }
    pub fn get_output_idl_path(&self) -> &Path {
        self.output_idl_path.as_path()
    }
}

impl CanisterInfoFactory for AssetsCanisterInfo {
    fn supports(info: &CanisterInfo) -> bool {
        info.get_type() == "assets"
    }

    fn create(info: &CanisterInfo) -> DfxResult<AssetsCanisterInfo> {
        let build_root = info.get_build_root();
        let name = info.get_name();

        let output_root = build_root.join(name);
        let output_wasm_path = output_root
            .join(Path::new("assetstorage.wasm"));
        let output_idl_path = output_wasm_path.with_extension("did");

        Ok(AssetsCanisterInfo {
            output_wasm_path,
            output_idl_path,
        })
    }
}
