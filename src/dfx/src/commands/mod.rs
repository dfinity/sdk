use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use clap::ArgMatches;

mod bootstrap;
mod build;
mod cache;
mod canister;
mod config;
mod identity;
mod language_service;
mod new;
mod ping;
mod replica;
mod start;
mod stop;
mod upgrade;

pub type CliExecFn = fn(&dyn Environment, &ArgMatches<'_>) -> DfxResult;
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
    pub fn execute(self: &CliCommand, env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult {
        (self.executor)(env, args)
    }
}

/// Returns all builtin commands understood by DFx.
pub fn builtin() -> Vec<CliCommand> {
    vec![
        CliCommand::new(bootstrap::construct(), bootstrap::exec),
        CliCommand::new(build::construct(), build::exec),
        CliCommand::new(cache::construct(), cache::exec),
        CliCommand::new(canister::construct(), canister::exec),
        CliCommand::new(config::construct(), config::exec),
        CliCommand::new(identity::construct(), identity::exec),
        CliCommand::new(language_service::construct(), language_service::exec),
        CliCommand::new(new::construct(), new::exec),
        CliCommand::new(ping::construct(), ping::exec),
        CliCommand::new(replica::construct(), replica::exec),
        CliCommand::new(start::construct(), start::exec),
        CliCommand::new(stop::construct(), stop::exec),
        CliCommand::new(upgrade::construct(), upgrade::exec),
    ]
}
