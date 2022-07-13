use crate::config::dfinity::CanisterTypeProperties;
use crate::lib::canister_info::{CanisterInfo, CanisterInfoFactory};
use crate::lib::error::DfxResult;
use anyhow::{bail, Context};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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
    fn create(info: &CanisterInfo) -> DfxResult<Self> {
        #[derive(Deserialize)]
        struct Project {
            target_directory: PathBuf,
        }
        let metadata = Command::new("cargo")
            .args(["metadata", "--no-deps", "--format-version=1", "--locked"])
            .stderr(Stdio::inherit())
            .stdout(Stdio::piped())
            .output()
            .context("Failed to run `cargo metadata`")?;
        if !metadata.status.success() {
            bail!("`cargo metadata` was unsuccessful");
        }
        let Project { target_directory } = serde_json::from_slice(&metadata.stdout)
            .context("Failed to read metadata from `cargo metadata`")?;
        let (package, candid) =
            if let CanisterTypeProperties::Rust { package, candid } = info.type_specific.clone() {
                (package, candid)
            } else {
                bail!(
                    "Attempted to construct a custom canister from a type:{} canister config",
                    info.type_specific.name()
                );
            };

        let workspace_root = info.get_workspace_root();
        let output_wasm_path =
            target_directory.join(format!("wasm32-unknown-unknown/release/{package}.wasm"));
        let candid = if let Some(remote_candid) = info.get_remote_candid_if_remote() {
            remote_candid
        } else {
            candid
        };
        let output_idl_path = workspace_root.join(candid);

        Ok(Self {
            package,
            output_wasm_path,
            output_idl_path,
        })
    }
}
