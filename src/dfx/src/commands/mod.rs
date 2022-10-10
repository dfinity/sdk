use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::Subcommand;

mod beta;
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
mod nns;
mod ping;
mod quickstart;
mod remote;
mod replica;
mod schema;
mod sns;
mod start;
mod stop;
mod toolchain;
mod upgrade;
mod wallet;

#[derive(Subcommand)]
pub enum Command {
    #[clap(hide(true))]
    Beta(beta::BetaOpts),
    Bootstrap(bootstrap::BootstrapOpts),
    Build(build::CanisterBuildOpts),
    Cache(cache::CacheOpts),
    Canister(canister::CanisterOpts),
    Deploy(deploy::DeployOpts),
    Diagnose(diagnose::DiagnoseOpts),
    Fix(fix::FixOpts),
    Generate(generate::GenerateOpts),
    Identity(identity::IdentityOpts),
    Info(info::InfoOpts),
    #[clap(name("_language-service"))]
    LanguageServices(language_service::LanguageServiceOpts),
    Ledger(ledger::LedgerOpts),
    New(new::NewOpts),
    Nns(nns::NnsOpts),
    Ping(ping::PingOpts),
    Quickstart,
    Remote(remote::RemoteOpts),
    Replica(replica::ReplicaOpts),
    Schema(schema::SchemaOpts),
    #[clap(hide(true))]
    Sns(sns::SnsOpts),
    Start(start::StartOpts),
    Stop(stop::StopOpts),
    Toolchain(toolchain::ToolchainOpts),
    Upgrade(upgrade::UpgradeOpts),
    Wallet(wallet::WalletOpts),
}

pub fn exec(env: &dyn Environment, cmd: Command) -> DfxResult {
    match cmd {
        Command::Beta(v) => beta::exec(env, v),
        Command::Bootstrap(v) => bootstrap::exec(env, v),
        Command::Build(v) => build::exec(env, v),
        Command::Cache(v) => cache::exec(env, v),
        Command::Canister(v) => canister::exec(env, v),
        Command::Deploy(v) => deploy::exec(env, v),
        Command::Diagnose(v) => diagnose::exec(env, v),
        Command::Fix(v) => fix::exec(env, v),
        Command::Generate(v) => generate::exec(env, v),
        Command::Identity(v) => identity::exec(env, v),
        Command::Info(v) => info::exec(env, v),
        Command::LanguageServices(v) => language_service::exec(env, v),
        Command::Ledger(v) => ledger::exec(env, v),
        Command::New(v) => new::exec(env, v),
        Command::Nns(v) => nns::exec(env, v),
        Command::Ping(v) => ping::exec(env, v),
        Command::Quickstart => quickstart::exec(env),
        Command::Remote(v) => remote::exec(env, v),
        Command::Replica(v) => replica::exec(env, v),
        Command::Schema(v) => schema::exec(env, v),
        Command::Sns(v) => sns::exec(env, v),
        Command::Start(v) => start::exec(env, v),
        Command::Stop(v) => stop::exec(env, v),
        Command::Toolchain(v) => toolchain::exec(env, v),
        Command::Upgrade(v) => upgrade::exec(env, v),
        Command::Wallet(v) => wallet::exec(env, v),
    }
}
