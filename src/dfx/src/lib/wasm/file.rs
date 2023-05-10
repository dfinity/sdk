use crate::lib::error::DfxResult;

use anyhow::Context;
use flate2::read::GzDecoder;
use std::io::Read;
use std::path::Path;

const WASM_HEADER: [u8; 4] = *b"\0asm";

pub fn is_wasm_format(bytes: &[u8]) -> bool {
    bytes.len() >= 4 && bytes[..4] == WASM_HEADER
}

// pub fn is_gzipped_wasm(bytes: &[u8]) -> bool {
//     let mut d = GzDecoder::new(&bytes[..]);
//     let mut buffer = [0; 4];
//     match d.read_exact(&mut buffer) {
//         Ok(()) => buffer == WASM_HEADER,
//         Err(_) => false,
//     }
// }

// pub fn read_wasm_module(bytes: &[u8]) -> DfxResult<walrus::Module> {
//     let bytes: Vec<u8> = dfx_core::fs::read(path).context("Failed to read wasm")?;

//     let module = ic_wasm::utils::parse_wasm(&bytes, true)
//         .with_context(|| format!("Failed to parse wasm file {}", path.display()))?;
//     Ok(module)
// }

pub fn read_wasm_module(path: &Path) -> DfxResult<walrus::Module> {
    let bytes: Vec<u8> = dfx_core::fs::read(path).context("Failed to read wasm")?;

    let unzipped_bytes = if is_wasm_format(&bytes) {
        bytes
    } else {
        let mut d = GzDecoder::new(&bytes[..]);
        let mut unzipped_bytes = vec![];
        d.read_to_end(&mut unzipped_bytes)
            .with_context(|| format!("Failed to decode gzipped file {}", path.display()))?;
        unzipped_bytes
    };

    let module = ic_wasm::utils::parse_wasm(&unzipped_bytes, true)
        .with_context(|| format!("Failed to parse wasm file {}", path.display()))?;
    Ok(module)
}
