use crate::commands::CliResult;
use console::{style, Term};

pub mod logo;

pub fn fake_command<F: FnOnce() -> CliResult>(command_impl: F) -> CliResult {
    Term::stderr().write_line(
        format!("{}", style(">>> This is a mocked command.").red().bold()).as_str()
    )?;

    command_impl()
}
