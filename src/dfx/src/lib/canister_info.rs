#![allow(dead_code)]
use crate::config::dfinity::Config;
use crate::lib::error::{DfxError, DfxResult};
use ic_http_agent::{Blob, CanisterId};
use rand::{thread_rng, RngCore};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// Information about a canister project (source code, destination, etc).
#[derive(Debug)]
pub struct CanisterInfo {
    name: String,
    input_path: PathBuf,

    output_root: PathBuf,
    idl_path: PathBuf,

    output_wasm_path: PathBuf,
    output_idl_path: PathBuf,
    output_did_js_path: PathBuf,
    output_canister_js_path: PathBuf,
    output_assets_root: PathBuf,

    canister_id: RefCell<Option<CanisterId>>,
    canister_id_path: PathBuf,

    has_frontend: bool,
}

impl CanisterInfo {
    pub fn load(config: &Config, name: &str) -> DfxResult<CanisterInfo> {
        let workspace_root = config.get_path().parent().unwrap();
        let build_root = workspace_root.join(
            config
                .get_config()
                .get_defaults()
                .get_build()
                .get_output("build/"),
        );
        let idl_path = build_root.join("idl/");

        let canister_map = (&config.get_config().canisters).as_ref().ok_or_else(|| {
            DfxError::Unknown("No canisters in the configuration file.".to_string())
        })?;

        let canister_config = canister_map
            .get(name)
            .ok_or_else(|| DfxError::CannotFindCanisterName(name.to_string()))?;
        let main_path = PathBuf::from_str(canister_config.main.as_ref().ok_or_else(|| {
            DfxError::Unknown("Main field mandatory for canister config.".to_string())
        })?)
        .expect("Could not convert Main field to a path.");

        let has_frontend = canister_config.frontend.is_some();

        let input_path = workspace_root.join(&main_path);
        let output_root = build_root.join(name);
        let output_wasm_path = output_root
            .join(
                main_path
                    .file_name()
                    .ok_or_else(|| DfxError::Unknown("Main is not a file path.".to_string()))?,
            )
            .with_extension("wasm");
        let output_idl_path = output_wasm_path.with_extension("did");
        let output_did_js_path = output_wasm_path.with_extension("did.js");
        let output_canister_js_path = output_wasm_path.with_extension("js");
        let output_assets_root = output_root.join("assets");

        let canister_id_path = output_root.join("_canister.id");

        Ok(CanisterInfo {
            name: name.to_string(),
            input_path,

            output_root,
            idl_path,
            output_wasm_path,
            output_idl_path,
            output_did_js_path,
            output_canister_js_path,
            output_assets_root,

            canister_id: RefCell::new(None),
            canister_id_path,

            has_frontend,
        })
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }
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
    pub fn get_idl_dir_path(&self) -> &Path {
        self.idl_path.as_path()
    }
    pub fn get_idl_file_path(&self) -> Option<PathBuf> {
        let idl_path = self.get_idl_dir_path();
        let canister_id = self.get_canister_id()?;
        Some(
            idl_path
                .join(canister_id.to_text().split_off(3))
                .with_extension("did"),
        )
    }
    pub fn get_canister_id_path(&self) -> &Path {
        self.canister_id_path.as_path()
    }

    pub fn get_canister_id(&self) -> Option<CanisterId> {
        let canister_id = self.canister_id.replace(None).or_else(|| {
            std::fs::read(&self.canister_id_path)
                .ok()
                .map(|cid| CanisterId::from(Blob::from(cid)))
        });

        self.canister_id.replace(canister_id.clone());

        canister_id
    }

    pub fn has_frontend(&self) -> bool {
        self.has_frontend
    }

    pub fn generate_canister_id(&self) -> DfxResult<CanisterId> {
        let mut rng = thread_rng();
        let mut v: Vec<u8> = Vec::with_capacity(8);
        rng.fill_bytes(v.as_mut_slice());

        Ok(CanisterId::from(Blob(v)))
    }
}
