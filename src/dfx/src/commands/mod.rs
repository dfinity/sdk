use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use anyhow::bail;
use clap::Subcommand;

mod beta;
mod build;
mod cache;
mod canister;
mod deploy;
mod deps;
mod diagnose;
mod extension;
mod fix;
mod generate;
mod identity;
mod info;
mod language_service;
mod ledger;
mod new;
mod ping;
mod quickstart;
mod remote;
mod schema;
mod start;
mod stop;
mod toolchain;
mod upgrade;
mod wallet;

#[derive(Subcommand)]
pub enum DfxCommand {
    #[command(hide = true)]
    Beta(beta::BetaOpts),
    Build(build::CanisterBuildOpts),
    Cache(cache::CacheOpts),
    Canister(canister::CanisterOpts),
    Deploy(deploy::DeployOpts),
    Deps(deps::DepsOpts),
    Diagnose(diagnose::DiagnoseOpts),
    Fix(fix::FixOpts),
    #[command(hide = true)]
    Extension(extension::ExtensionOpts),
    Generate(generate::GenerateOpts),
    Identity(identity::IdentityOpts),
    Info(info::InfoOpts),
    #[command(name = "_language-service")]
    LanguageServices(language_service::LanguageServiceOpts),
    Ledger(ledger::LedgerOpts),
    New(new::NewOpts),
    Ping(ping::PingOpts),
    Quickstart(quickstart::QuickstartOpts),
    Remote(remote::RemoteOpts),
    Schema(schema::SchemaOpts),
    Start(start::StartOpts),
    Stop(stop::StopOpts),
    Toolchain(toolchain::ToolchainOpts),
    Upgrade(upgrade::UpgradeOpts),
    Wallet(wallet::WalletOpts),
}

pub fn exec(env: &dyn Environment, cmd: DfxCommand) -> DfxResult {
    match cmd {
        DfxCommand::Beta(v) => beta::exec(env, v),
        DfxCommand::Build(v) => build::exec(env, v),
        DfxCommand::Cache(v) => cache::exec(env, v),
        DfxCommand::Canister(v) => canister::exec(env, v),
        DfxCommand::Deploy(v) => deploy::exec(env, v),
        DfxCommand::Deps(v) => deps::exec(env, v),
        DfxCommand::Diagnose(v) => diagnose::exec(env, v),
        DfxCommand::Fix(v) => fix::exec(env, v),
        DfxCommand::Extension(v) => extension::exec(env, v),
        DfxCommand::Generate(v) => generate::exec(env, v),
        DfxCommand::Identity(v) => identity::exec(env, v),
        DfxCommand::Info(v) => info::exec(env, v),
        DfxCommand::LanguageServices(v) => language_service::exec(env, v),
        DfxCommand::Ledger(v) => ledger::exec(env, v),
        DfxCommand::New(v) => new::exec(env, v),
        DfxCommand::Ping(v) => ping::exec(env, v),
        DfxCommand::Quickstart(v) => quickstart::exec(env, v),
        DfxCommand::Remote(v) => remote::exec(env, v),
        DfxCommand::Schema(v) => schema::exec(v),
        DfxCommand::Start(v) => start::exec(env, v),
        DfxCommand::Stop(v) => stop::exec(env, v),
        DfxCommand::Toolchain(v) => toolchain::exec(env, v),
        DfxCommand::Upgrade(v) => upgrade::exec(env, v),
        DfxCommand::Wallet(v) => wallet::exec(env, v),
    }
}

pub fn exec_without_env(cmd: DfxCommand) -> DfxResult {
    match cmd {
        DfxCommand::Schema(v) => schema::exec(v),
        _ => bail!("Cannot execute this command without environment."),
    }
}
