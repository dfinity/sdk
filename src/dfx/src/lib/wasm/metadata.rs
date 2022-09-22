use crate::lib::error::DfxResult;
use crate::lib::metadata::names::CANDID_SERVICE;

use anyhow::Context;
use fn_error_context::context;
use ic_wasm::metadata::{add_metadata, Kind};
use std::path::Path;

#[context("Failed to add candid service metadata from {} to {}.", idl_path.to_string_lossy(), wasm_path.to_string_lossy())]
pub fn add_candid_service_metadata(wasm_path: &Path, idl_path: &Path) -> DfxResult {
    let wasm = std::fs::read(wasm_path).context("Could not read the WASM module.")?;
    let idl = std::fs::read(&idl_path)
        .with_context(|| format!("Failed to read {}", idl_path.to_string_lossy()))?;
    let processed_wasm = add_metadata(&wasm, Kind::Public, CANDID_SERVICE, idl)
        .context("Could not add metadata to the WASM module.")?;
    std::fs::write(wasm_path, &processed_wasm)
        .with_context(|| format!("Could not write WASM to {:?}", wasm_path))?;
    Ok(())
}
