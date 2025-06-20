use anyhow::bail;
use clap::Args;
use ic_utils::interfaces::management_canister::builders::{
    CanisterUpgradeOptions, InstallMode, WasmMemoryPersistence,
};

use crate::lib::error::DfxResult;

/// CLI options for the mode of canister installation.
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

#[derive(Debug, Clone, PartialEq)]
pub enum InstallModeHint {
    Install,
    Reinstall,
    Upgrade(Option<CanisterUpgradeOptions>),
    Auto(Option<CanisterUpgradeOptions>),
}

enum HighLevelMode {
    Install,
    Reinstall,
    Upgrade,
    Auto,
}

impl InstallModeOpt {
    /// `dfx canister install` defaults to 'install' mode.
    pub fn mode_for_canister_install(&self) -> DfxResult<InstallModeHint> {
        self.resolve_install_mode(HighLevelMode::Install)
    }

    /// `dfx deploy` defaults to 'auto' mode.
    pub fn mode_for_deploy(&self) -> DfxResult<InstallModeHint> {
        self.resolve_install_mode(HighLevelMode::Auto)
    }

    fn resolve_install_mode(&self, default_mode: HighLevelMode) -> DfxResult<InstallModeHint> {
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

        let high_level_mode = match self.mode.as_deref() {
            Some("install") => HighLevelMode::Install,
            Some("reinstall") => HighLevelMode::Reinstall,
            Some("upgrade") => HighLevelMode::Upgrade,
            Some("auto") => HighLevelMode::Auto,
            None => default_mode,
            _ => unreachable!(),
        };

        if canister_upgrade_options.is_some()
            && matches!(
                high_level_mode,
                HighLevelMode::Install | HighLevelMode::Reinstall
            )
        {
            bail!("--skip-pre-upgrade and --wasm-memory-persistence can only be used with mode 'upgrade' or 'auto'.");
        }
        match high_level_mode {
            HighLevelMode::Install => Ok(InstallModeHint::Install),
            HighLevelMode::Reinstall => Ok(InstallModeHint::Reinstall),
            HighLevelMode::Upgrade => Ok(InstallModeHint::Upgrade(canister_upgrade_options)),
            HighLevelMode::Auto => Ok(InstallModeHint::Auto(canister_upgrade_options)),
        }
    }
}

impl InstallModeHint {
    pub fn to_install_mode_with_wasm_path(&self) -> DfxResult<InstallMode> {
        match self {
            InstallModeHint::Install => Ok(InstallMode::Install),
            InstallModeHint::Reinstall => Ok(InstallMode::Reinstall),
            InstallModeHint::Upgrade(opt) => Ok(InstallMode::Upgrade(*opt)),
            InstallModeHint::Auto(_) => bail!("The install mode cannot be auto when using --wasm"),
        }
    }

    pub fn to_install_mode(
        &self,
        upgrade_in_auto: bool,
        wasm_memory_persistence_embedded: Option<WasmMemoryPersistence>,
    ) -> InstallMode {
        match self {
            InstallModeHint::Install => InstallMode::Install,
            InstallModeHint::Reinstall => InstallMode::Reinstall,
            InstallModeHint::Upgrade(opt) => InstallMode::Upgrade(*opt),
            InstallModeHint::Auto(opt) => {
                let opt = if opt.is_none() && wasm_memory_persistence_embedded.is_some() {
                    Some(CanisterUpgradeOptions {
                        skip_pre_upgrade: None,
                        wasm_memory_persistence: wasm_memory_persistence_embedded,
                    })
                } else {
                    *opt
                };
                match upgrade_in_auto {
                    true => InstallMode::Upgrade(opt),
                    false => InstallMode::Install,
                }
            }
        }
    }
}
