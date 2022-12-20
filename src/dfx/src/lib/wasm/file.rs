use crate::lib::error::DfxResult;
use anyhow::Context;
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
