use crate::lib::{environment::Environment, error::DfxResult};

use anyhow::Error;
use clap::Parser;
use slog::info;
use sysinfo::{ProcessExt, System, SystemExt};

/// Kills all dfx-related processes on the system. Useful if a process gets stuck.
#[derive(Parser)]
pub struct KillallOpts;

pub fn exec(env: &dyn Environment, _: KillallOpts) -> DfxResult {
    let mut info = System::new();
    info.refresh_processes();
    let mut n = 0;
    let self_pid = sysinfo::get_current_pid().map_err(Error::msg)?;
    for (pid, proc) in info.processes() {
        if proc.exe().file_name().is_some_and(|name| {
            *pid != self_pid
                && (name == "dfx"
                    || name == "icx-proxy"
                    || name == "replica"
                    || name == "ic-ref"
                    || name == "ic-btc-adapter"
                    || name == "ic-https-outcalls-adapter"
                    || name == "ic-starter")
        }) {
            n += 1;
            proc.kill();
        }
    }
    info!(env.get_logger(), "Killed {n} processes");
    Ok(())
}
