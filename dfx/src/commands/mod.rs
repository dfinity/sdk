use crate::lib::error::DfxResult;
use clap::ArgMatches;

mod build;
mod call;
mod start;

pub type CliExecFn = fn(&ArgMatches<'_>) -> DfxResult;
pub struct CliCommand {
    subcommand: clap::App<'static, 'static>,
    executor: CliExecFn,
}

impl CliCommand {
    pub fn new(subcommand: clap::App<'static, 'static>, executor: CliExecFn) -> CliCommand {
        CliCommand {
            subcommand,
            executor,
        }
    }
    pub fn get_subcommand(&self) -> &clap::App<'static, 'static> {
        &self.subcommand
    }
    pub fn get_name(&self) -> &str {
        self.subcommand.get_name()
    }
    pub fn execute(self: &CliCommand, args: &ArgMatches<'_>) -> DfxResult {
        (self.executor)(args)
    }
}

pub fn builtin() -> Vec<CliCommand> {
    vec![
        CliCommand::new(build::construct(), build::exec),
        CliCommand::new(call::construct(), call::exec),
        CliCommand::new(start::construct(), start::exec),
    ]
}
