use crate::lib::canister_info::{CanisterInfo, CanisterInfoFactory};
use crate::lib::error::DfxResult;
use anyhow::{bail, Context};
use dfx_core::config::model::dfinity::CanisterTypeProperties;
use fn_error_context::context;
use std::path::{Path, PathBuf};

pub struct AssetsCanisterInfo {
    input_root: PathBuf,
    source_paths: Vec<PathBuf>,

    output_wasm_path: PathBuf,
    output_idl_path: PathBuf,
    build: Vec<String>,
}

impl AssetsCanisterInfo {
    pub fn get_source_paths(&self) -> Vec<PathBuf> {
        self.source_paths
            .iter()
            .map(|sp| self.input_root.join(sp))
            .collect::<_>()
    }
    pub fn get_output_idl_path(&self) -> &Path {
        self.output_idl_path.as_path()
    }
    pub fn get_build_tasks(&self) -> &[String] {
        &self.build
    }

    #[context("Failed to assert source paths.")]
    pub fn assert_source_paths(&self) -> DfxResult<()> {
        let source_paths = self.get_source_paths();
        let input_root = &self.input_root;
        let source_paths: Vec<PathBuf> = source_paths.iter().map(|x| input_root.join(x)).collect();
        for source_path in &source_paths {
            let canonical = dfx_core::fs::canonicalize(source_path).with_context(|| {
                format!(
                    "Unable to determine canonical location of asset source path {}",
                    source_path.to_string_lossy()
                )
            })?;
            if !canonical.starts_with(input_root) {
                bail!(
                    "Directory at '{}' is outside the workspace root.",
                    source_path.to_path_buf().display()
                );
            }
        }
        Ok(())
    }
}

impl CanisterInfoFactory for AssetsCanisterInfo {
    fn create(info: &CanisterInfo) -> DfxResult<Self> {
        let input_root = info.get_workspace_root().to_path_buf();
        // If there are no "source" field, we just ignore this.
        let (source_paths, build) =
            if let CanisterTypeProperties::Assets { source, build } = info.type_specific.clone() {
                (source, build.into_vec())
            } else {
                bail!(
                    "Attempted to construct an assets canister from a type:{} canister config",
                    info.type_specific.name()
                )
            };

        let output_root = info.get_output_root();

        let output_wasm_path = output_root.join(Path::new("assetstorage.wasm.gz"));
        let output_idl_path = output_wasm_path.with_extension("").with_extension("did");

        Ok(AssetsCanisterInfo {
            input_root,
            source_paths,
            output_wasm_path,
            output_idl_path,
            build,
        })
    }
}
