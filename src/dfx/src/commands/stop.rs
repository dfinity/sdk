use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::Parser;
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
fn kill_all(system: &System, proc: &Process) {
    let processes = list_all_descendants(system, proc);
    for proc in processes {
        proc.kill_with(Signal::Term);
    }
}

pub fn exec(env: &dyn Environment, _opts: StopOpts) -> DfxResult {
    let pid_file_path = env.get_temp_dir().join("pid");
    if pid_file_path.exists() {
        // Read and verify it's not running. If it is just return.
        if let Ok(s) = std::fs::read_to_string(&pid_file_path) {
            if let Ok(pid) = s.parse::<Pid>() {
                let mut system = System::new();
                system.refresh_processes();
                if let Some(proc) = system.process(pid) {
                    kill_all(&system, proc);
                }
            }
        }
    } else {
        eprintln!("No local network replica found. Nothing to do.");
    }

    // We ignore errors here because there is no effect for the user. We're just being nice.
    let _ = std::fs::remove_file(&pid_file_path);

    Ok(())
}
