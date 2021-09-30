use crate::lib::canister_info::{CanisterInfo, CanisterInfoFactory};
use crate::lib::error::DfxResult;
use std::path::{Path, PathBuf};

pub struct RustCanisterInfo {
    package: String,
    idl_path: PathBuf,
}

impl RustCanisterInfo {
    pub fn get_package(&self) -> &str {
        &self.package
    }

    pub fn get_idl_path(&self) -> &Path {
        self.idl_path.as_path()
    }
}

impl CanisterInfoFactory for RustCanisterInfo {
    fn supports(info: &CanisterInfo) -> bool {
        info.get_type() == "rust"
    }

    fn create(info: &CanisterInfo) -> DfxResult<Self> {
        let package = info.get_extra::<String>("package")?;

        let workspace_root = info.get_workspace_root();
        let idl_path = workspace_root.join(info.get_extra::<PathBuf>("candid")?);

        Ok(Self { package, idl_path })
    }
}
