use crate::lib::error::DfxResult;
use anyhow::Context;
use fn_error_context::context;
use slog::info;
use std::io::Read;

include!(concat!(env!("OUT_DIR"), "/load_assets.rs"));

pub fn dfinity_logo() -> String {
    let colors = supports_color::on(atty::Stream::Stdout);
    if let Some(colors) = colors {
        //Some terminals, notably MacOS's Terminal.app, do not support Truecolor (RGB-colored characters) properly.
        //Therefore we use xterm256 coloring when the program is running in such a terminal.
        if colors.has_16m {
            return include_str!("../../assets/dfinity-color.aart").to_string();
        } else if colors.has_256 {
            return include_str!("../../assets/dfinity-color-xterm256.aart").to_string();
        }
    }
    include_str!("../../assets/dfinity-nocolor.aart").to_string()
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

#[context("Failed to load assets wasm.")]
pub fn assets_wasm(logger: &slog::Logger) -> DfxResult<Vec<u8>> {
    let mut wasm = Vec::new();

    if let Ok(dfx_assets_wasm) = std::env::var("DFX_ASSETS_WASM") {
        info!(logger, "Using wasm at path: {}", dfx_assets_wasm);
        std::fs::File::open(&dfx_assets_wasm)
            .with_context(|| format!("Failed to open {}.", dfx_assets_wasm))?
            .read_to_end(&mut wasm)
            .with_context(|| format!("Failed to read file content for {}.", dfx_assets_wasm))?;
    } else {
        let mut canister_assets =
            assetstorage_canister().context("Failed to load asset canister archive.")?;
        for file in canister_assets
            .entries()
            .context("Failed to read asset canister archive entries.")?
        {
            let mut file = file.context("Failed to read asset canister archive entry.")?;
            if file
                .header()
                .path()
                .context("Failed to read archive entry path.")?
                .ends_with("assetstorage.wasm.gz")
            {
                file.read_to_end(&mut wasm)
                    .context("Failed to read archive entry.")?;
            }
        }
    }
    Ok(wasm)
}
