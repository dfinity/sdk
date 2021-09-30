use crate::lib::canister_info::{CanisterInfo, CanisterInfoFactory};
use crate::lib::error::DfxResult;
use std::path::{Path, PathBuf};

pub struct RustCanisterInfo {
    package: String,
    output_wasm_path: PathBuf,
    output_idl_path: PathBuf,
}

impl RustCanisterInfo {
    pub fn get_package(&self) -> &str {
        &self.package
    }

    pub fn get_output_wasm_path(&self) -> &Path {
        self.output_wasm_path.as_path()
    }

    pub fn get_output_idl_path(&self) -> &Path {
        self.output_idl_path.as_path()
    }
}

impl CanisterInfoFactory for RustCanisterInfo {
    fn supports(info: &CanisterInfo) -> bool {
        info.get_type() == "rust"
    }

    fn create(info: &CanisterInfo) -> DfxResult<Self> {
        let package = info.get_extra::<String>("package")?;

        let workspace_root = info.get_workspace_root();
        let output_wasm_path = workspace_root.join(format!(
            "target/wasm32-unknown-unknown/release/{}.wasm",
            package
        ));
        let output_idl_path = workspace_root.join(info.get_extra::<PathBuf>("candid")?);

        Ok(Self {
            package,
            output_wasm_path,
            output_idl_path,
        })
    }
}
