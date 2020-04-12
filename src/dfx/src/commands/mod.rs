use crate::config::dfinity;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::message::UserMessage;

use clap::{ArgMatches, Clap};

mod bootstrap;
mod build;
mod cache;
mod canister;
mod config;
mod language_service;
mod new;
mod replica;
mod start;
mod stop;
mod upgrade;

/// Deprecated.
pub struct CliCommand {
    subcommand: clap::App<'static>,
    executor: CliExecFn,
}

/// Deprecated.
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

/// Deprecated.
pub type CliExecFn = fn(&dyn Environment, &ArgMatches) -> DfxResult;

/// DFX commands.
#[derive(Clap, Clone, Debug)]
pub enum Command {
    /// Bootstrap command.
    #[clap(about = UserMessage::BootstrapCommand.to_str(), name = "bootstrap")]
    Bootstrap(dfinity::ConfigDefaultsBootstrap),

    /// Build command.
    #[clap(about = UserMessage::BuildCommand.to_str(), name = "build")]
    Build,

    /// Cache command.
    #[clap(about = UserMessage::CacheCommand.to_str(), name = "cache")]
    Cache,

    /// Canister command.
    #[clap(about = UserMessage::CanisterCommand.to_str(), name = "canister")]
    Canister,

    /// Config command.
    #[clap(about = UserMessage::ConfigCommand.to_str(), name = "config")]
    Config,

    /// IDE command.
    #[clap(about = UserMessage::IDECommand.to_str(), name = "_language-service")]
    IDE,

    /// New command.
    #[clap(about = UserMessage::NewCommand.to_str(), name = "new")]
    New,

    /// Replica command.
    #[clap(about = UserMessage::ReplicaCommand.to_str(), name = "replica")]
    Replica,

    /// Start command.
    #[clap(about = UserMessage::StartCommand.to_str(), name = "start")]
    Start,

    /// Stop command.
    #[clap(about = UserMessage::StopCommand.to_str(), name = "stop")]
    Stop,

    /// Upgrade command.
    #[clap(about = UserMessage::UpgradeCommand.to_str(), name = "upgrade")]
    Upgrade,
}

/// Execute a DFX command.
pub fn exec(env: &dyn Environment, cmd: Command) -> DfxResult {
    match cmd {
        Command::Bootstrap(cfg) => bootstrap::exec(env, &cfg),
        // TODO: Clean up remaining commands.
        Command::Build => build::exec(env, &build::construct().get_matches()),
        Command::Cache => cache::exec(env, &cache::construct().get_matches()),
        Command::Canister => canister::exec(env, &canister::construct().get_matches()),
        Command::Config => config::exec(env, &config::construct().get_matches()),
        Command::IDE => language_service::exec(env, &language_service::construct().get_matches()),
        Command::New => new::exec(env, &new::construct().get_matches()),
        Command::Replica => replica::exec(env, &replica::construct().get_matches()),
        Command::Start => start::exec(env, &start::construct().get_matches()),
        Command::Stop => stop::exec(env, &stop::construct().get_matches()),
        Command::Upgrade => upgrade::exec(env, &upgrade::construct().get_matches()),
    }
}
