use crate::lib::{environment::Environment, error::DfxResult};

use anyhow::Error;
use clap::Parser;
use dfx_core::config::cache::get_bin_cache_root;
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
    // first, kill the dfx processes, so they can't restart the child processes
    for proc in info.processes_by_exact_name("dfx") {
        if proc.pid() != self_pid {
            n += 1;
            proc.kill();
        }
    }
    // then, kill anything that was installed alongside dfx
    info.refresh_processes();
    let versions_dir = get_bin_cache_root()?;
    for (pid, proc) in info.processes() {
        if *pid != self_pid && proc.exe().starts_with(&versions_dir) {
            n += 1;
            proc.kill();
        }
    }
    info!(env.get_logger(), "Killed {n} processes");
    Ok(())
}
