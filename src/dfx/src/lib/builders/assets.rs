use crate::config::cache::Cache;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::builders::{BuildConfig, BuildOutput, CanisterBuilder, IdlBuildOutput, WasmBuildOutput};
use crate::lib::models::canister::CanisterPool;
use crate::lib::canister_info::CanisterInfo;
use std::path::Path;
use crate::util;
use std::sync::Arc;

pub struct AssetsBuilder {
    _cache: Arc<dyn Cache>,
}

impl AssetsBuilder {
    pub fn new(env: &dyn Environment) -> DfxResult<Self> {
        Ok(AssetsBuilder {
            _cache: env.get_cache(),
        })
    }
}

impl CanisterBuilder for AssetsBuilder {
    fn supports(&self, info: &CanisterInfo) -> bool {
        info.get_type() == "assets"
    }

    fn build(&self, _pool: &CanisterPool, info: &CanisterInfo, _config: &BuildConfig) -> DfxResult<BuildOutput> {
        let mut canister_assets = util::assets::assetstorage_canister()?;
        for file in canister_assets.entries()? {
            let mut file = file?;

            if file.header().entry_type().is_dir() {
                continue;
            }
            file.unpack_in(info.get_output_root())?;
        }

        let wasm_path = info.get_output_root().join(Path::new("assetstorage.wasm"));
        let idl_path = info.get_output_root().join(Path::new("assetstorage.did"));
        Ok(BuildOutput {
            canister_id: info
                .get_canister_id()
                .expect("Could not find canister ID."),
            wasm: WasmBuildOutput::File(wasm_path),
            idl: IdlBuildOutput::File(idl_path),
        })
    }
}
