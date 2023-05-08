use crate::lib::error::DfxResult;
use anyhow::Context;
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn is_wasm_format(path: &Path) -> DfxResult<bool> {
    let mut file =
        File::open(path).with_context(|| format!("Failed to open {}", path.display()))?;
    let mut header = [0; 4];
    file.read_exact(&mut header)?;
    Ok(header == *b"\0asm")
}

pub fn read_wasm_module(path: &Path) -> DfxResult<walrus::Module> {
    let bytes = dfx_core::fs::read(path).context("Failed to read wasm")?;

    let unzipped_bytes = if is_wasm_format(path)? {
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
