use crate::lib::error::DfxResult;
use anyhow::Context;
use fn_error_context::context;
use slog::info;
use std::io::Read;

include!(concat!(env!("OUT_DIR"), "/load_assets.rs"));

pub fn dfinity_logo() -> String {
    if atty::is(atty::Stream::Stdout) {
        //MacOS's Terminal.app does not support Truecolor (RGB-colored characters) properly.
        //Therefore we use xterm256 coloring when the program is running on macos
        if std::env::consts::OS == "macos" {
            include_str!("../../assets/dfinity-color-xterm256.aart").to_string()
        } else {
            include_str!("../../assets/dfinity-color.aart").to_string()
        }
    } else {
        include_str!("../../assets/dfinity-nocolor.aart").to_string()
    }
}

#[context("Failed to load wallet wasm.")]
pub fn wallet_wasm(logger: &slog::Logger) -> DfxResult<Vec<u8>> {
    let mut wasm = Vec::new();

    if let Ok(dfx_wallet_wasm) = std::env::var("DFX_WALLET_WASM") {
        info!(logger, "Using wasm at path: {}", dfx_wallet_wasm);
        std::fs::File::open(&dfx_wallet_wasm)
            .with_context(|| format!("Failed to open {}.", dfx_wallet_wasm))?
            .read_to_end(&mut wasm)
            .with_context(|| format!("Failed to read file content for {}.", dfx_wallet_wasm))?;
    } else {
        let mut canister_assets =
            wallet_canister().context("Failed to load wallet canister archive.")?;
        for file in canister_assets
            .entries()
            .context("Failed to read wallet canister archive entries.")?
        {
            let mut file = file.context("Failed to read wallet canister archive entry.")?;
            if file
                .header()
                .path()
                .context("Failed to read archive entry path.")?
                .ends_with("wallet.wasm")
            {
                file.read_to_end(&mut wasm)
                    .context("Failed to read archive entry.")?;
            }
        }
    }
    Ok(wasm)
}
