use crate::lib::canister_info::{CanisterInfo, CanisterInfoFactory};
use crate::lib::error::DfxResult;

use anyhow::bail;
use std::path::{Path, PathBuf};

pub struct AssetsCanisterInfo {
    input_root: PathBuf,
    source_paths: Vec<PathBuf>,

    output_wasm_path: PathBuf,
    output_idl_path: PathBuf,
    output_assets_path: PathBuf,
}

impl AssetsCanisterInfo {
    pub fn get_source_paths(&self) -> &Vec<PathBuf> {
        &self.source_paths
    }
    pub fn get_output_wasm_path(&self) -> &Path {
        self.output_wasm_path.as_path()
    }
    pub fn get_output_idl_path(&self) -> &Path {
        self.output_idl_path.as_path()
    }
    pub fn get_output_assets_path(&self) -> &Path {
        self.output_assets_path.as_path()
    }

    pub fn assert_source_paths(&self) -> DfxResult<()> {
        let source_paths = self.get_source_paths();
        let input_root = &self.input_root;
        let source_paths: Vec<PathBuf> = source_paths.iter().map(|x| input_root.join(x)).collect();
        for source_path in &source_paths {
            let canonical = source_path.canonicalize()?;
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
    fn supports(info: &CanisterInfo) -> bool {
        info.get_type() == "assets"
    }

    fn create(info: &CanisterInfo) -> DfxResult<AssetsCanisterInfo> {
        let build_root = info.get_build_root();
        let name = info.get_name();

        let input_root = info.get_workspace_root().to_path_buf();
        // If there are no "source" field, we just ignore this.
        let source_paths = if info.has_extra("source") {
            info.get_extra::<Vec<PathBuf>>("source")?
        } else {
            vec![]
        };

        let output_root = build_root.join(name);

        let output_wasm_path = output_root.join(Path::new("assetstorage.wasm"));
        let output_idl_path = output_wasm_path.with_extension("did");
        let output_assets_path = output_root.join(Path::new("assets"));

        Ok(AssetsCanisterInfo {
            input_root,
            source_paths,
            output_wasm_path,
            output_idl_path,
            output_assets_path,
        })
    }
}
