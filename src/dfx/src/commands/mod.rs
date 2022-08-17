use crate::lib::error::DfxResult;
use crate::{init_env, BaseOpts};

use clap::{Args, Subcommand};

mod bootstrap;
mod build;
mod cache;
mod canister;
mod deploy;
mod diagnose;
mod fix;
mod generate;
mod identity;
mod info;
mod language_service;
mod ledger;
mod new;
mod ping;
mod remote;
mod replica;
mod schema;
mod start;
mod stop;
mod toolchain;
mod upgrade;
mod wallet;

#[derive(Subcommand)]
pub enum Command {
    Bootstrap(BaseOpts<bootstrap::BootstrapOpts>),
    Build(BaseOpts<build::CanisterBuildOpts>),
    Cache(cache::CacheCommand),
    Canister(canister::CanisterCommand),
    Deploy(BaseOpts<deploy::DeployOpts>),
    Diagnose(BaseOpts<diagnose::DiagnoseOpts>),
    Fix(BaseOpts<fix::FixOpts>),
    Generate(BaseOpts<generate::GenerateOpts>),
    Identity(identity::IdentityCommand),
    Info(BaseOpts<info::InfoOpts>),
    #[clap(name("_language-service"))]
    LanguageServices(BaseOpts<language_service::LanguageServiceOpts>),
    Ledger(ledger::LedgerCommand),
    New(BaseOpts<new::NewOpts>),
    Ping(BaseOpts<ping::PingOpts>),
    Remote(remote::RemoteCommand),
    Replica(BaseOpts<replica::ReplicaOpts>),
    Schema(BaseOpts<schema::SchemaOpts>),
    Start(BaseOpts<start::StartOpts>),
    Stop(BaseOpts<stop::StopOpts>),
    Toolchain(toolchain::ToolchainCommand),
    Upgrade(BaseOpts<upgrade::UpgradeOpts>),
    Wallet(wallet::WalletCommand),
}

#[derive(Args)]
pub struct NetworkOpts<T: Args> {
    #[clap(flatten)]
    base_opts: BaseOpts<T>,
    /// Override the compute network to connect to. By default, the local network is used.
    ///
    /// A valid URL (starting with `http:` or `https:`) can be used here, and a special
    /// ephemeral network will be created specifically for this request. E.g.
    /// "http://localhost:12345/" is a valid network name.
    #[clap(long)]
    network: Option<String>,
}

pub fn dispatch(cmd: Command) -> DfxResult {
    match cmd {
        Command::Cache(v) => cache::dispatch(v),
        Command::Canister(v) => canister::dispatch(v),
        Command::Identity(v) => identity::dispatch(v),
        Command::Ledger(v) => ledger::dispatch(v),
        Command::Remote(v) => remote::dispatch(v),
        Command::Toolchain(v) => toolchain::dispatch(v),
        Command::Wallet(v) => wallet::dispatch(v),

        Command::Bootstrap(v) => bootstrap::exec(&init_env(v.env_opts)?, v.command_opts),
        Command::Build(v) => build::exec(&init_env(v.env_opts)?, v.command_opts),
        Command::Deploy(v) => deploy::exec(&init_env(v.env_opts)?, v.command_opts),
        Command::Diagnose(v) => diagnose::exec(&init_env(v.env_opts)?, v.command_opts),
        Command::Fix(v) => fix::exec(&init_env(v.env_opts)?, v.command_opts),
        Command::Generate(v) => generate::exec(&init_env(v.env_opts)?, v.command_opts),
        Command::Info(v) => info::exec(&init_env(v.env_opts)?, v.command_opts),
        Command::LanguageServices(v) => {
            language_service::exec(&init_env(v.env_opts)?, v.command_opts)
        }
        Command::New(v) => new::exec(&init_env(v.env_opts)?, v.command_opts),
        Command::Ping(v) => ping::exec(&init_env(v.env_opts)?, v.command_opts),
        Command::Replica(v) => replica::exec(&init_env(v.env_opts)?, v.command_opts),
        Command::Schema(v) => schema::exec(&init_env(v.env_opts)?, v.command_opts),
        Command::Start(v) => start::exec(&init_env(v.env_opts)?, v.command_opts),
        Command::Stop(v) => stop::exec(&init_env(v.env_opts)?, v.command_opts),
        Command::Upgrade(v) => upgrade::exec(&init_env(v.env_opts)?, v.command_opts),
    }
}
