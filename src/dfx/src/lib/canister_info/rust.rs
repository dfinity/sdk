use crate::lib::canister_info::{CanisterInfo, CanisterInfoFactory};
use crate::lib::error::DfxResult;
use std::path::PathBuf;

/// TODO: refactor this into a build manifest, see issue #
pub struct RustCanisterInfo {
    pub wasm_path: PathBuf,
    pub idl_path: PathBuf,
}

impl CanisterInfoFactory for RustCanisterInfo {
    fn supports(info: &CanisterInfo) -> bool {
        info.get_type() == "rust"
    }

    fn create(canister_info: &CanisterInfo) -> DfxResult<Self> {
        let idl_path = canister_info.get_extra::<PathBuf>("candid")?;
        let idl_path = canister_info.get_workspace_root().join(idl_path);

        let wasm_path = canister_info.get_extra::<PathBuf>("output")?;
        let wasm_path = canister_info.get_workspace_root().join(wasm_path);

        Ok(RustCanisterInfo {
            wasm_path,
            idl_path,
        })
    }
}
