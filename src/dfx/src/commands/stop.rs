use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::provider::{get_network_descriptor, LocalBindDetermination};

use anyhow::bail;
use clap::Parser;
use garcon::{Delay, Waiter};
use sysinfo::{Pid, Process, ProcessExt, Signal, System, SystemExt};

/// Stops the local network replica.
#[derive(Parser)]
pub struct StopOpts {}

fn list_all_descendants<'a>(system: &'a System, proc: &'a Process) -> Vec<&'a Process> {
    let mut result = Vec::new();

    for process in system.processes().values() {
        if let Some(ppid) = process.parent() {
            if ppid == proc.pid() {
                result.extend(list_all_descendants(system, process));
            }
        }
    }
    result.push(proc);

    result
}

/// Recursively kill a process and ALL its children.
fn kill_all(system: &System, proc: &Process) -> Vec<Pid> {
    let processes = list_all_descendants(system, proc);
    for proc in &processes {
        proc.kill_with(Signal::Term);
    }
    processes.iter().map(|proc| proc.pid()).collect()
}

fn wait_until_all_exited(mut system: System, mut pids: Vec<Pid>) -> DfxResult {
    let mut waiter = Delay::builder()
        .timeout(std::time::Duration::from_secs(30))
        .throttle(std::time::Duration::from_secs(1))
        .build();
    waiter.start();

    loop {
        system.refresh_processes();

        pids.retain(|&pid| system.process(pid).is_some());

        if pids.is_empty() {
            return Ok(());
        }
        if waiter.wait().is_err() {
            let remaining = pids
                .iter()
                .map(|pid| format!("{pid}"))
                .collect::<Vec<_>>()
                .join(" ");
            bail!("Failed to kill all processes.  Remaining: {remaining}");
        }
    }
}

pub fn exec(env: &dyn Environment, _opts: StopOpts) -> DfxResult {
    let network_descriptor =
        get_network_descriptor(env.get_config(), None, LocalBindDetermination::AsConfigured)?;
    let pid_file_path = network_descriptor.local_server_descriptor()?.dfx_pid_path();
    if pid_file_path.exists() {
        // Read and verify it's not running. If it is just return.
        if let Ok(s) = std::fs::read_to_string(&pid_file_path) {
            if let Ok(pid) = s.parse::<Pid>() {
                let mut system = System::new();
                system.refresh_processes();
                let pids_killed = if let Some(proc) = system.process(pid) {
                    kill_all(&system, proc)
                } else {
                    vec![]
                };
                wait_until_all_exited(system, pids_killed)?;
            }
        }
    } else {
        eprintln!("No local network replica found. Nothing to do.");
    }

    // We ignore errors here because there is no effect for the user. We're just being nice.
    let _ = std::fs::remove_file(&pid_file_path);

    Ok(())
}
