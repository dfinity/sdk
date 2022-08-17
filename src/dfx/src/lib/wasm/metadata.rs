use crate::lib::error::DfxResult;
use crate::lib::metadata::names::CANDID_SERVICE;

use anyhow::Context;
use fn_error_context::context;
use ic_wasm::metadata::{add_metadata, Kind};
use std::path::Path;

#[context("Failed to add candid service metadata from {} to {}.", idl_path.to_string_lossy(), wasm_path.to_string_lossy())]
pub fn add_candid_service_metadata(wasm_path: &Path, idl_path: &Path) -> DfxResult {
    let mut m = walrus::ModuleConfig::new()
        .parse_file(wasm_path)
        .with_context(|| format!("Failed to parse file {}", wasm_path.to_string_lossy()))?;
    let idl = std::fs::read(&idl_path)
        .with_context(|| format!("Failed to read {}", idl_path.to_string_lossy()))?;

    add_metadata(&mut m, Kind::Public, CANDID_SERVICE, idl);

    m.emit_wasm_file(wasm_path)
        .with_context(|| format!("Failed to emit wasm to {}", wasm_path.to_string_lossy()))?;
    Ok(())
}
