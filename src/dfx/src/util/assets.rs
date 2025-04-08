use crate::lib::error::DfxResult;
use anyhow::{bail, Context};
use fn_error_context::context;
use slog::info;
use std::io::Read;

include!(concat!(env!("OUT_DIR"), "/load_assets.rs"));

#[context("Failed to load wallet wasm.")]
pub fn wallet_wasm(logger: &slog::Logger) -> DfxResult<Vec<u8>> {
    if let Ok(dfx_wallet_wasm) = std::env::var("DFX_WALLET_WASM") {
        info!(logger, "Using wasm at path: {}", dfx_wallet_wasm);
        Ok(dfx_core::fs::read(dfx_wallet_wasm.as_ref())?)
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
                .ends_with("wallet.wasm.gz")
            {
                let mut wasm = vec![];
                file.read_to_end(&mut wasm)
                    .context("Failed to read archive entry.")?;
                return Ok(wasm);
            }
        }
        bail!("Failed to find wallet canister archive entry.");
    }
}

#[context("Failed to load assets wasm.")]
pub fn assets_wasm(logger: &slog::Logger) -> DfxResult<Vec<u8>> {
    if let Ok(dfx_assets_wasm) = std::env::var("DFX_ASSETS_WASM") {
        info!(logger, "Using wasm at path: {}", dfx_assets_wasm);
        Ok(dfx_core::fs::read(dfx_assets_wasm.as_ref())?)
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
                let mut wasm = vec![];
                file.read_to_end(&mut wasm)
                    .context("Failed to read archive entry.")?;
                return Ok(wasm);
            }
        }
        bail!("Failed to find asset canister archive entry.");
    }
}

#[allow(unused)]
#[context("Failed to load bitcoin wasm.")]
pub fn bitcoin_wasm(logger: &slog::Logger) -> DfxResult<Vec<u8>> {
    if let Ok(dfx_assets_wasm) = std::env::var("DFX_BITCOIN_WASM") {
        info!(logger, "Using wasm at path: {}", dfx_assets_wasm);
        Ok(dfx_core::fs::read(dfx_assets_wasm.as_ref())?)
    } else {
        let mut canister_assets =
            btc_canister().context("Failed to load bitcoin canister archive.")?;
        for file in canister_assets
            .entries()
            .context("Failed to read bitcoin canister archive entries.")?
        {
            let mut file = file.context("Failed to read bitcoin canister archive entry.")?;
            if file
                .header()
                .path()
                .context("Failed to read archive entry path.")?
                .ends_with("ic-btc-canister.wasm.gz")
            {
                let mut wasm = vec![];
                file.read_to_end(&mut wasm)
                    .context("Failed to read archive entry.")?;
                return Ok(wasm);
            }
        }
        bail!("Failed to find bitcoin canister archive entry");
    }
}

pub fn management_idl() -> DfxResult<String> {
    // FIXME get idl from replica when it's available
    // The ic.did file is downloaded in assets/build.rs.
    // The git rev is specified in portal_rev.txt.
    let did = include_str!(concat!(env!("OUT_DIR"), "/ic.did"));
    Ok(did.to_string())
}
