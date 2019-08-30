extern crate hyper;

use crate::lib::error::DfxResult;
use clap::ArgMatches;

mod build;
mod config;
mod new;
mod send;
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

fn add_builtin(
    v: &mut Vec<CliCommand>,
    available: bool,
    subcommand: clap::App<'static, 'static>,
    executor: CliExecFn,
) {
    if available {
        v.push(CliCommand::new(subcommand, executor));
    }
}

pub fn builtin() -> Vec<CliCommand> {
    let mut v: Vec<CliCommand> = Vec::new();

    add_builtin(&mut v, build::available(), build::construct(), build::exec);
    add_builtin(
        &mut v,
        config::available(),
        config::construct(),
        config::exec,
    );
    add_builtin(&mut v, new::available(), new::construct(), new::exec);
    add_builtin(&mut v, true, send::construct(), send::exec);
    add_builtin(&mut v, start::available(), start::construct(), start::exec);

    v
}
