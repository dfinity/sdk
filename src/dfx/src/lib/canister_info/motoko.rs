use crate::lib::canister_info::{CanisterInfo, CanisterInfoFactory};
use crate::lib::error::DfxResult;
use std::path::{Path, PathBuf};

pub struct MotokoCanisterInfo {
    input_path: PathBuf,
    output_root: PathBuf,
    idl_path: PathBuf,

    output_wasm_path: PathBuf,
    output_idl_path: PathBuf,
    output_did_js_path: PathBuf,
    output_canister_js_path: PathBuf,
    output_assets_root: PathBuf,

    packtool: Option<String>,
    has_frontend: bool,
}

impl MotokoCanisterInfo {
    pub fn get_main_path(&self) -> &Path {
        self.input_path.as_path()
    }
    pub fn get_output_wasm_path(&self) -> &Path {
        self.output_wasm_path.as_path()
    }
    pub fn get_output_idl_path(&self) -> &Path {
        self.output_idl_path.as_path()
    }
    pub fn get_output_did_js_path(&self) -> &Path {
        self.output_did_js_path.as_path()
    }
    pub fn get_output_canister_js_path(&self) -> &Path {
        self.output_canister_js_path.as_path()
    }
    pub fn get_output_assets_root(&self) -> &Path {
        self.output_assets_root.as_path()
    }
    pub fn get_output_root(&self) -> &Path {
        self.output_root.as_path()
    }

    pub fn get_packtool(&self) -> &Option<String> {
        &self.packtool
    }

    pub fn has_frontend(&self) -> bool {
        self.has_frontend
    }
}

impl CanisterInfoFactory for MotokoCanisterInfo {
    fn supports(info: &CanisterInfo) -> bool {
        info.get_type() == "motoko"
    }

    fn create(info: &CanisterInfo) -> DfxResult<MotokoCanisterInfo> {
        let workspace_root = info.get_workspace_root();
        let build_root = info.get_build_root();
        let name = info.get_name();
        let idl_path = build_root.join("idl/");

        let main_path = info.get_extra::<PathBuf>("main")?;

        let input_path = workspace_root.join(&main_path);
        let output_root = build_root.join(name);
        let output_wasm_path = output_root.join(name).with_extension("wasm");
        let output_idl_path = output_wasm_path.with_extension("did");
        let output_did_js_path = output_wasm_path.with_extension("did.js");
        let output_canister_js_path = output_wasm_path.with_extension("js");
        let output_assets_root = output_root.join("assets");

        Ok(MotokoCanisterInfo {
            input_path,
            output_root,
            idl_path,
            output_wasm_path,
            output_idl_path,
            output_did_js_path,
            output_canister_js_path,
            output_assets_root,
            packtool: info.get_packtool().clone(),
            has_frontend: info.get_extra_value("frontend").is_some(),
        })
    }
}
