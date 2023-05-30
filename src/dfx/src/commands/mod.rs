use std::ffi::OsString;

use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use anyhow::bail;
use clap::Subcommand;
use lazy_static::lazy_static;

mod beta;
mod bootstrap;
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
pub mod nns;
mod ping;
mod quickstart;
mod remote;
mod replica;
mod schema;
pub mod sns;
mod start;
mod stop;
mod toolchain;
mod upgrade;
mod wallet;

lazy_static! {
    pub static ref DEFAULT_COMMANDS: Vec<String> = include_str!("mod.rs")
        .split("pub enum DfxCommand {")
        .nth(2)
        .unwrap()
        .split('}').next()
        .unwrap()
        .lines()
        .filter(|l| !l.trim().starts_with(r#"//"#) && !l.trim().is_empty())
        .map(|variant| variant
            .split('(').next()
            .unwrap_or_default()
            .trim()
            .to_lowercase())
        .map(|s| s.replace("languageservices", "_language-services"))
        .collect();
}

#[derive(Subcommand)]
pub enum DfxCommand {
    #[command(hide = true)]
    Beta(beta::BetaOpts),
    Bootstrap(bootstrap::BootstrapOpts),
    Build(build::CanisterBuildOpts),
    Cache(cache::CacheOpts),
    Canister(canister::CanisterOpts),
    Deploy(deploy::DeployOpts),
    #[command(hide = true)]
    Deps(deps::DepsOpts),
    Diagnose(diagnose::DiagnoseOpts),
    Fix(fix::FixOpts),
    Extension(extension::ExtensionOpts),
    // Executes an extension
    #[command(external_subcommand)]
    ExtensionRun(Vec<OsString>),
    Generate(generate::GenerateOpts),
    Identity(identity::IdentityOpts),
    Info(info::InfoOpts),
    #[command(name = "_language-service")]
    LanguageServices(language_service::LanguageServiceOpts),
    Ledger(ledger::LedgerOpts),
    New(new::NewOpts),
    Nns(nns::NnsOpts),
    Ping(ping::PingOpts),
    Quickstart(quickstart::QuickstartOpts),
    Remote(remote::RemoteOpts),
    Replica(replica::ReplicaOpts),
    Schema(schema::SchemaOpts),
    Sns(sns::SnsOpts),
    Start(start::StartOpts),
    Stop(stop::StopOpts),
    Toolchain(toolchain::ToolchainOpts),
    Upgrade(upgrade::UpgradeOpts),
    Wallet(wallet::WalletOpts),
}

pub fn exec(env: &dyn Environment, cmd: DfxCommand) -> DfxResult {
    match cmd {
        DfxCommand::Beta(v) => beta::exec(env, v),
        DfxCommand::Bootstrap(v) => bootstrap::exec(env, v),
        DfxCommand::Build(v) => build::exec(env, v),
        DfxCommand::Cache(v) => cache::exec(env, v),
        DfxCommand::Canister(v) => canister::exec(env, v),
        DfxCommand::Deploy(v) => deploy::exec(env, v),
        DfxCommand::Deps(v) => deps::exec(env, v),
        DfxCommand::Diagnose(v) => diagnose::exec(env, v),
        DfxCommand::Fix(v) => fix::exec(env, v),
        DfxCommand::Extension(v) => extension::exec(env, v),
        DfxCommand::ExtensionRun(v) => extension::run::exec(env, v.into()),
        DfxCommand::Generate(v) => generate::exec(env, v),
        DfxCommand::Identity(v) => identity::exec(env, v),
        DfxCommand::Info(v) => info::exec(env, v),
        DfxCommand::LanguageServices(v) => language_service::exec(env, v),
        DfxCommand::Ledger(v) => ledger::exec(env, v),
        DfxCommand::New(v) => new::exec(env, v),
        DfxCommand::Nns(v) => nns::exec(env, v),
        DfxCommand::Ping(v) => ping::exec(env, v),
        DfxCommand::Quickstart(v) => quickstart::exec(env, v),
        DfxCommand::Remote(v) => remote::exec(env, v),
        DfxCommand::Replica(v) => replica::exec(env, v),
        DfxCommand::Schema(v) => schema::exec(v),
        DfxCommand::Sns(v) => sns::exec(env, v),
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

#[test]
fn test_name() {
    let dc = DEFAULT_COMMANDS.clone();
    static DEFAULT_COMMANDS_LEN: usize = 28;
    assert_eq!(
        dc.len(),
        DEFAULT_COMMANDS_LEN,
        "You probably added or removed subcomman. Adjust DEFAULT_COMMANDS_LEN"
    );
    let commands = vec![
        "beta",
        "bootstrap",
        "build",
        "cache",
        "canister",
        "deploy",
        "deps",
        "diagnose",
        "fix",
        "extension",
        "generate",
        "identity",
        "info",
        "_language-services",
        "ledger",
        "new",
        "nns",
        "ping",
        "quickstart",
        "remote",
        "replica",
        "schema",
        "sns",
        "start",
        "stop",
        "toolchain",
        "upgrade",
        "wallet",
    ]
    .into_iter()
    .map(String::from)
    .collect::<Vec<_>>();
    assert_eq!(commands, dc);
}
