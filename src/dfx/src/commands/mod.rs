use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use clap::Clap;

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
        Command::Bootstrap(v) => bootstrap::exec(env, v),
        Command::Build(v) => build::exec(env, v),
        Command::Cache(v) => cache::exec(env, v),
        Command::Canister(v) => canister::exec(env, v),
        Command::Config(v) => config::exec(env, v),
        Command::Deploy(v) => deploy::exec(env, v),
        Command::Identity(v) => identity::exec(env, v),
        Command::LanguageServices(v) => language_service::exec(env, v),
        Command::New(v) => new::exec(env, v),
        Command::Ping(v) => ping::exec(env, v),
        Command::Replica(v) => replica::exec(env, v),
        Command::Start(v) => start::exec(env, v),
        Command::Stop(v) => stop::exec(env, v),
        Command::Upgrade(v) => upgrade::exec(env, v),
    }
}
