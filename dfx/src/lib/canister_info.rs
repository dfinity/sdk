#![allow(dead_code)]
use crate::config::dfinity::Config;
use crate::lib::error::{DfxError, DfxResult};
use ic_http_agent::{Blob, CanisterId};
use rand::{thread_rng, Rng};
use std::cell::RefCell;
use std::ops::Shl;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

/// Information about a canister project (source code, destination, etc).
#[derive(Debug)]
pub struct CanisterInfo {
    name: String,
    input_path: PathBuf,

    output_root: PathBuf,

    output_wasm_path: PathBuf,
    output_idl_path: PathBuf,
    output_js_path: PathBuf,

    canister_id: RefCell<Option<CanisterId>>,
    canister_id_path: PathBuf,
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
        let output_js_path = output_wasm_path.with_extension("js");

        let canister_id_path = output_root.join("_canister.id");

        Ok(CanisterInfo {
            name: name.to_string(),
            input_path,

            output_root,
            output_wasm_path,
            output_idl_path,
            output_js_path,

            canister_id: RefCell::new(None),
            canister_id_path,
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
    pub fn get_output_js_path(&self) -> &Path {
        self.output_js_path.as_path()
    }
    pub fn get_output_root(&self) -> &Path {
        self.output_root.as_path()
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

    pub fn generate_canister_id(&self) -> DfxResult<CanisterId> {
        // Generate a random u64.
        let time_since_the_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards.");
        let cid = u64::from(time_since_the_epoch.as_millis() as u32).shl(32)
            + u64::from(thread_rng().gen::<u32>());

        Ok(CanisterId::from(cid))
    }
}
