use anyhow::bail;
use clap::Args;
use ic_utils::interfaces::management_canister::builders::{
    CanisterUpgradeOptions, InstallMode, WasmMemoryPersistence,
};

use crate::lib::error::DfxResult;

/// CLI options for the mode of installing canister.
///
/// Reused in `dfx canister install` and `dfx deploy`.
#[derive(Args, Clone, Debug, Default)]
pub struct InstallModeOpt {
    /// Specifies the mode of canister installation.
    ///
    /// If set to 'auto', either 'install' or 'upgrade' will be used, depending on whether the canister is already installed.
    #[arg(long, short, value_parser = ["install", "reinstall", "upgrade", "auto"])]
    mode: Option<String>,

    /// Skip the pre_upgrade hook on upgrade.
    ///
    /// This requires the mode to be set to 'upgrade' or 'auto'.
    #[arg(long)]
    skip_pre_upgrade: bool,

    /// Keep or replace the Wasm main memory on upgrade.
    ///
    /// This requires the mode to be set to 'upgrade' or 'auto'.
    #[arg(long, value_parser = ["keep", "replace"])]
    wasm_memory_persistence: Option<String>,
}

impl InstallModeOpt {
    /// `dfx canister install` defaults to 'install' mode.
    pub fn mode_for_canister_install(&self) -> DfxResult<Option<InstallMode>> {
        self.resolve_install_mode(Some(InstallMode::Install))
    }

    /// `dfx deploy` defaults to 'auto' mode.
    pub fn mode_for_deploy(&self) -> DfxResult<Option<InstallMode>> {
        self.resolve_install_mode(None)
    }

    fn resolve_install_mode(&self, default: Option<InstallMode>) -> DfxResult<Option<InstallMode>> {
        let wasm_memory_persistence = match self.wasm_memory_persistence {
            Some(ref s) => match s.as_str() {
                "keep" => Some(WasmMemoryPersistence::Keep),
                "replace" => Some(WasmMemoryPersistence::Replace),
                _ => unreachable!(),
            },
            None => None,
        };
        let canister_upgrade_options = match (self.skip_pre_upgrade, wasm_memory_persistence) {
            (false, None) => None,
            (s, w) => Some(CanisterUpgradeOptions {
                skip_pre_upgrade: Some(s),
                wasm_memory_persistence: w,
            }),
        };
        if canister_upgrade_options.is_some() {
            if self.mode.as_deref() == Some("upgrade")
                || self.mode.as_deref() == Some("auto")
                || self.mode.is_none()
            {
                Ok(Some(InstallMode::Upgrade(canister_upgrade_options)))
            } else {
                bail!("--skip-pre-upgrade and --wasm-memory-persistence can only be used with mode 'upgrade' or 'auto'.");
            }
        } else {
            match self.mode.as_deref() {
                Some("install") => Ok(Some(InstallMode::Install)),
                Some("reinstall") => Ok(Some(InstallMode::Reinstall)),
                Some("upgrade") => Ok(Some(InstallMode::Upgrade(None))),
                Some("auto") => Ok(None),
                None => Ok(default),
                _ => unreachable!(),
            }
        }
    }
}
