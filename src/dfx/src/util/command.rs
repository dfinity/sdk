use crate::lib::error::DfxResult;
use anyhow::Context;
use std::path::Path;
use std::process::Command;

pub fn direct_or_shell_command(s: &str, cwd: &Path) -> DfxResult<Command> {
    let words = shell_words::split(s).with_context(|| format!("Cannot parse command '{}'.", s))?;
    let canonical_result = dfx_core::fs::canonicalize(&cwd.join(&words[0]));
    let mut cmd = if words.len() == 1 && canonical_result.is_ok() {
        // If the command is a file, execute it directly.
        let file = canonical_result.unwrap();
        Command::new(file)
    } else {
        // Execute the command in `sh -c` to allow pipes.
        let mut sh_cmd = Command::new("sh");
        sh_cmd.args(["-c", s]);
        sh_cmd
    };
    cmd.current_dir(cwd);
    Ok(cmd)
}
