use crate::lib::error::DfxResult;

use anyhow::Context;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use fn_error_context::context;
use std::io::{Read, Write};
use std::path::Path;

const WASM_HEADER: [u8; 4] = *b"\0asm";

pub fn is_wasm_format(bytes: &[u8]) -> bool {
    bytes.len() >= 4 && bytes[..4] == WASM_HEADER
}

pub fn read_wasm_module(path: &Path) -> DfxResult<walrus::Module> {
    let bytes: Vec<u8> = dfx_core::fs::read(path).context("Failed to read wasm")?;

    let unzipped_bytes = if is_wasm_format(&bytes) {
        bytes
    } else {
        decompress_bytes(&bytes)?
    };

    let module = ic_wasm::utils::parse_wasm(&unzipped_bytes, true)
        .with_context(|| format!("Failed to parse wasm file {}", path.display()))?;
    Ok(module)
}

#[context("Failed to parse bytes as wasm")]
pub fn bytes_to_module(bytes: &[u8]) -> DfxResult<walrus::Module> {
    Ok(ic_wasm::utils::parse_wasm(bytes, true)?)
}

#[context("Failed to encode bytes as gzip")]
pub fn compress_bytes(bytes: &[u8]) -> DfxResult<Vec<u8>> {
    let mut e = GzEncoder::new(Vec::new(), Compression::default());
    e.write_all(bytes)?;
    Ok(e.finish()?)
}

#[context("Failed to decode bytes as gzip")]
pub fn decompress_bytes(bytes: &[u8]) -> DfxResult<Vec<u8>> {
    let mut d = GzDecoder::new(bytes);
    let mut unzipped_bytes = vec![];
    d.read_to_end(&mut unzipped_bytes)?;
    Ok(unzipped_bytes)
}
