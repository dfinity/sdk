use crate::lib::canister_info::{CanisterInfo, CanisterInfoFactory};
use crate::lib::error::DfxResult;
use anyhow::{bail, ensure, Context};
use cargo_metadata::Metadata;
use dfx_core::config::model::dfinity::CanisterTypeProperties;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub struct RustCanisterInfo {
    package: String,
    output_wasm_path: PathBuf,
}

impl RustCanisterInfo {
    pub fn get_package(&self) -> &str {
        &self.package
    }

    pub fn get_output_wasm_path(&self) -> &Path {
        self.output_wasm_path.as_path()
    }
}

impl CanisterInfoFactory for RustCanisterInfo {
    fn create(info: &CanisterInfo) -> DfxResult<Self> {
        let metadata = Command::new("cargo")
            .args(["metadata", "--no-deps", "--format-version=1", "--locked"])
            .stderr(Stdio::inherit())
            .stdout(Stdio::piped())
            .output()
            .context("Failed to run `cargo metadata`")?;
        if !metadata.status.success() {
            bail!("`cargo metadata` was unsuccessful");
        }

        let package = if let CanisterTypeProperties::Rust { package, candid: _ } =
            info.type_specific.clone()
        {
            package
        } else {
            bail!(
                "Attempted to construct a custom canister from a type:{} canister config",
                info.type_specific.name()
            );
        };
        let metadata: Metadata = serde_json::from_slice(&metadata.stdout)
            .context("Failed to read metadata from `cargo metadata`")?;
        let package_info = metadata
            .packages
            .iter()
            .find(|x| x.name == package)
            .with_context(|| format!("No package `{package}` found"))?;
        let target = if let Some(exact_match_target) =
            package_info.targets.iter().find(|x| x.name == package)
        {
            ensure!(
                exact_match_target
                    .crate_types
                    .iter()
                    .any(|c| c == "cdylib" || c == "bin"),
                "Crate `{package}` is not a bin or cdylib",
            );
            exact_match_target
        } else {
            let mut candidate_targets = package_info
                .targets
                .iter()
                .filter(|x| x.crate_types.iter().any(|c| c == "cdylib" || c == "bin"));
            let target = candidate_targets
                .next()
                .with_context(|| format!("No bin or cdylib crates found in package `{package}`"))?;
            ensure!(
                candidate_targets.next().is_none(),
                "More than one bin/cdylib crate was found in package `{package}`",
            );
            target
        };

        let wasm_name = target.name.replace('-', "_");
        let output_wasm_path = metadata
            .target_directory
            .join(format!("wasm32-unknown-unknown/release/{wasm_name}.wasm"))
            .into();

        Ok(Self {
            package,
            output_wasm_path,
        })
    }
}
