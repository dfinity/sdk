use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::message::UserMessage;
use clap::{App, ArgMatches, SubCommand};
use sysinfo::{Pid, Process, ProcessExt, Signal, System, SystemExt};

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("stop").about(UserMessage::StopNode.to_str())
}

fn list_all_descendants(pid: Pid) -> Vec<Pid> {
    let mut result: Vec<Pid> = Vec::new();
    let system = System::new();

    for process in system.get_process_list().values() {
        if let Some(ppid) = process.parent() {
            if ppid == pid {
                result.append(list_all_descendants(process.pid()).as_mut());
            }
        }
    }
    result.push(pid);

    result
}

/// Recursively kill a process and ALL its children.
fn kill_all(pid: Pid) -> DfxResult {
    eprintln!("kill pid {} and all descendants", pid);
    let processes = list_all_descendants(pid);
    eprintln!(" - descendants: {:?}", processes);
    for pid in processes {
        eprintln!(" - kill process {}", pid);

        let process = Process::new(pid, None, 0);
        process.kill(Signal::Term);
    }

    Ok(())
}

pub fn exec(env: &dyn Environment, _args: &ArgMatches<'_>) -> DfxResult {
    let pid_file_path = env.get_temp_dir().join("pid");
    if pid_file_path.exists() {
        // Read and verify it's not running. If it is just return.
        if let Ok(s) = std::fs::read_to_string(&pid_file_path) {
            if let Ok(pid) = s.parse::<i32>() {
                kill_all(pid)?;
            }
        }
    } else {
        eprintln!("No local network replica found. Nothing to do.");
    }

    // We ignore errors here because there is no effect for the user. We're just being nice.
    let _ = std::fs::remove_file(&pid_file_path);

    Ok(())
}
