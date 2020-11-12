use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use clap::{ArgMatches, Clap};

mod bootstrap;
mod build;
mod cache;
mod canister;
mod config;
mod deploy;
mod identity;
mod language_service;
mod new;
mod ping;
mod replica;
mod start;
mod stop;
mod upgrade;

pub type CliExecFn = fn(&dyn Environment, &ArgMatches) -> DfxResult;
pub struct CliCommand {
    subcommand: clap::App<'static>,
    executor: CliExecFn,
}

impl CliCommand {
    pub fn new(subcommand: clap::App<'static>, executor: CliExecFn) -> CliCommand {
        CliCommand {
            subcommand,
            executor,
        }
    }
    pub fn get_subcommand(&self) -> &clap::App<'static> {
        &self.subcommand
    }
    pub fn get_name(&self) -> &str {
        self.subcommand.get_name()
    }
    pub fn execute(self: &CliCommand, env: &dyn Environment, args: &ArgMatches) -> DfxResult {
        (self.executor)(env, args)
    }
}

#[derive(Clap)]
pub enum Command {
    Bootstrap(bootstrap::BootstrapOpts),
    Build(build::CanisterBuildOpts),
    Cache(cache::CacheOpts),
    Canister(canister::CanisterOpts),
    Config(config::ConfigOpts),
    Deploy(deploy::DeployOpts),
    Identity(identity::IdentityOpt),
    LanguageServices(language_service::LanguageServiceOpts),
    New(new::NewOpts),
    Ping(ping::PingOpts),
    Replica(replica::ReplicaOpts),
    Start(start::StartOpts),
    Stop(stop::StopOpts),
    Upgrade(upgrade::UpgradeOpts),
}

pub fn exec(env: &dyn Environment, cmd: Command) -> DfxResult {
    match cmd {
        Command::Bootstrap(_v) => bootstrap::exec(env, &bootstrap::construct().get_matches()),
        Command::Build(_v) => build::exec(env, &build::construct().get_matches()),
        Command::Cache(_v) => cache::exec(env, &cache::construct().get_matches()),
        Command::Canister(_v) => canister::exec(env, &canister::construct().get_matches()),
        Command::Config(_v) => config::exec(env, &config::construct().get_matches()),
        Command::Deploy(_v) => deploy::exec(env, &deploy::construct().get_matches()),
        Command::Identity(_v) => identity::exec(env, &identity::construct().get_matches()),
        Command::LanguageServices(_v) => language_service::exec(env, &language_service::construct().get_matches()),
        Command::New(_v) => new::exec(env, &new::construct().get_matches()),
        Command::Ping(_v) => ping::exec(env, &ping::construct().get_matches()),
        Command::Replica(_v) => replica::exec(env, &replica::construct().get_matches()),
        Command::Start(_v) => start::exec(env, &start::construct().get_matches()),
        Command::Stop(_v) => stop::exec(env, &stop::construct().get_matches()),
        Command::Upgrade(_v) => upgrade::exec(env, &upgrade::construct().get_matches()),
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
        CliCommand::new(deploy::construct(), deploy::exec),
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
