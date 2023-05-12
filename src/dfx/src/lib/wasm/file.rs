use crate::lib::error::DfxResult;

use anyhow::{bail, Context};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use fn_error_context::context;
use std::io::{Read, Write};
use std::path::Path;

/// Read wasm module
///
/// Based on the file extension, it may decompress the file before parse the wasm module.
pub fn read_wasm_module(path: &Path) -> DfxResult<walrus::Module> {
    let bytes: Vec<u8> = dfx_core::fs::read(path)?;

    let m = match path.extension() {
        Some(f) if f == "gz" => {
            let unzip_bytes = decompress_bytes(&bytes)
                .with_context(|| format!("Failed to decode gzip file {:?}", path))?;
            bytes_to_module(&unzip_bytes).with_context(|| {
                format!("Failed to parse wasm module from decompressed {:?}", path)
            })?
        }
        Some(f) if f == "wasm" => bytes_to_module(&bytes)
            .with_context(|| format!("Failed to parse wasm module from {:?}", path))?,
        _ => {
            bail!("{:?} is neither a wasm nor a wasm.gz file", path);
        }
    };
    Ok(m)
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
