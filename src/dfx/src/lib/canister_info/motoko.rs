use crate::lib::canister_info::{CanisterInfo, CanisterInfoFactory};
use crate::lib::error::DfxResult;
use anyhow::{ensure, Context};
use dfx_core::config::model::dfinity::CanisterTypeProperties;
use std::path::{Path, PathBuf};

pub struct MotokoCanisterInfo {
    input_path: PathBuf,
    output_root: PathBuf,

    output_wasm_path: PathBuf,
    output_stable_path: PathBuf,
    output_did_js_path: PathBuf,
    output_canister_js_path: PathBuf,
    output_assets_root: PathBuf,

    packtool: Option<String>,
    moc_args: Option<String>,

    workspace_root: PathBuf,
}

impl MotokoCanisterInfo {
    pub fn get_main_path(&self) -> &Path {
        self.input_path.as_path()
    }
    pub fn get_output_wasm_path(&self) -> &Path {
        self.output_wasm_path.as_path()
    }
    pub fn get_output_stable_path(&self) -> &Path {
        self.output_stable_path.as_path()
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
    pub fn get_args(&self) -> &Option<String> {
        &self.moc_args
    }
    pub fn get_workspace_root(&self) -> &Path {
        self.workspace_root.as_path()
    }
}

impl CanisterInfoFactory for MotokoCanisterInfo {
    fn create(info: &CanisterInfo) -> DfxResult<MotokoCanisterInfo> {
        let workspace_root = info.get_workspace_root();
        let name = info.get_name();
        ensure!(
            matches!(info.type_specific, CanisterTypeProperties::Motoko { .. }),
            "Attempted to construct a custom canister from a type:{} canister config",
            info.type_specific.name()
        );
        let main_path = info
            .get_main_file()
            .context("`main` attribute is required on Motoko canisters in dfx.json (and Motoko is the default canister type if not otherwise specified)")?;
        let input_path = workspace_root.join(main_path);
        let output_root = info.get_output_root().to_path_buf();
        let output_wasm_path = output_root.join(name).with_extension("wasm");
        let output_stable_path = output_wasm_path.with_extension("most");
        let output_did_js_path = output_wasm_path.with_extension("did.js");
        let output_canister_js_path = output_wasm_path.with_extension("js");
        let output_assets_root = output_root.join("assets");
        let workspace_root = workspace_root.to_path_buf();

        Ok(MotokoCanisterInfo {
            input_path,
            output_root,
            output_wasm_path,
            output_stable_path,
            output_did_js_path,
            output_canister_js_path,
            output_assets_root,
            packtool: info.get_packtool().clone(),
            moc_args: info.get_args().clone(),
            workspace_root,
        })
    }
}
