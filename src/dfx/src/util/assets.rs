use crate::lib::error::DfxResult;
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

pub fn wallet_wasm(logger: &slog::Logger) -> DfxResult<Vec<u8>> {
    let mut wasm = Vec::new();

    if let Ok(dfx_wallet_wasm) = std::env::var("DFX_WALLET_WASM") {
        info!(logger, "Using wasm at path: {}", dfx_wallet_wasm);
        std::fs::File::open(&dfx_wallet_wasm)?.read_to_end(&mut wasm)?;
    } else {
        let mut canister_assets = wallet_canister()?;
        for file in canister_assets.entries()? {
            let mut file = file?;
            if file.header().path()?.ends_with("wallet.wasm") {
                file.read_to_end(&mut wasm)?;
            }
        }
    }
    Ok(wasm)
}
