use std::collections::HashMap;
use std::ffi::OsString;
use std::fmt::Debug;

use crate::config::dfx_version;
use crate::lib::error::DfxResult;
use crate::lib::{environment::Environment, extension::manager::ExtensionManager};

use anyhow::bail;
use clap::{error::ErrorKind, ArgMatches, Args, Command, Error, FromArgMatches, Subcommand};

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

pub enum DfxCommand {
    Beta(beta::BetaOpts),
    Bootstrap(bootstrap::BootstrapOpts),
    Build(build::CanisterBuildOpts),
    Cache(cache::CacheOpts),
    Canister(canister::CanisterOpts),
    Deploy(deploy::DeployOpts),
    Deps(deps::DepsOpts),
    Diagnose(diagnose::DiagnoseOpts),
    Fix(fix::FixOpts),
    Extension(extension::ExtensionOpts),
    // Executes an extension
    ExtensionRun(Vec<OsString>),
    Generate(generate::GenerateOpts),
    Identity(identity::IdentityOpts),
    Info(info::InfoOpts),
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

impl Subcommand for DfxCommand {
    fn augment_subcommands(cmd: Command) -> Command {
        let extension_subcommands =
            if let Ok(ext_manager) = ExtensionManager::new(dfx_version(), false) {
                ext_manager
                    .list_installed_extensions()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|ext| ext.into_clap_command(&ext_manager))
                    .collect::<Vec<Command>>()
            } else {
                vec![]
            };
        let mut cmd = cmd
            .subcommand(beta::BetaOpts::augment_args(
                Command::new("beta").hide(true),
            ))
            .subcommand(bootstrap::BootstrapOpts::augment_args(Command::new(
                "bootstrap",
            )))
            .subcommand(build::CanisterBuildOpts::augment_args(Command::new(
                "build",
            )))
            .subcommand(cache::CacheOpts::augment_args(Command::new("cache")))
            .subcommand(canister::CanisterOpts::augment_args(Command::new(
                "canister",
            )))
            .subcommand(deploy::DeployOpts::augment_args(Command::new("deploy")))
            .subcommand(deps::DepsOpts::augment_args(
                Command::new("deps").hide(true),
            ))
            .subcommand(diagnose::DiagnoseOpts::augment_args(Command::new(
                "diagnose",
            )))
            .subcommand(fix::FixOpts::augment_args(Command::new("fix")))
            .subcommand(Command::new("extension-run").allow_external_subcommands(true))
            .subcommand(extension::ExtensionOpts::augment_args(Command::new(
                "extension",
            )))
            .subcommand(generate::GenerateOpts::augment_args(Command::new(
                "generate",
            )))
            .subcommand(identity::IdentityOpts::augment_args(Command::new(
                "identity",
            )))
            .subcommand(info::InfoOpts::augment_args(Command::new("info")))
            .subcommand(language_service::LanguageServiceOpts::augment_args(
                Command::new("_language-service"),
            ))
            .subcommand(ledger::LedgerOpts::augment_args(Command::new("ledger")))
            .subcommand(new::NewOpts::augment_args(Command::new("new")))
            .subcommand(nns::NnsOpts::augment_args(Command::new("nns")))
            .subcommand(ping::PingOpts::augment_args(Command::new("ping")))
            .subcommand(quickstart::QuickstartOpts::augment_args(Command::new(
                "quickstart",
            )))
            .subcommand(remote::RemoteOpts::augment_args(Command::new("remote")))
            .subcommand(replica::ReplicaOpts::augment_args(Command::new("replica")))
            .subcommand(sns::SnsOpts::augment_args(Command::new("sns")))
            .subcommand(schema::SchemaOpts::augment_args(Command::new("schema")))
            .subcommand(start::StartOpts::augment_args(Command::new("start")))
            // .subcommand(sns::SnsOpts::augment_args(Command::new("sns")))
            .subcommand(stop::StopOpts::augment_args(Command::new("stop")))
            .subcommand(toolchain::ToolchainOpts::augment_args(Command::new(
                "toolchain",
            )))
            .subcommand(upgrade::UpgradeOpts::augment_args(Command::new("upgrade")))
            .subcommand(wallet::WalletOpts::augment_args(Command::new("wallet")))
            .subcommands(extension_subcommands)
            .allow_external_subcommands(true)
            .subcommand_required(true);
        sort_clap_commands(&mut cmd);
        cmd
    }

    fn augment_subcommands_for_update(cmd: Command) -> Command {
        Self::augment_subcommands(cmd)
    }

    fn has_subcommand(name: &str) -> bool {
        // let f = include_str!("mod.rs");
        let extension_subcommands =
            if let Ok(ext_manager) = ExtensionManager::new(dfx_version(), false) {
                ext_manager
                    .list_installed_extensions()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|v| v.name)
                    .collect()
            } else {
                vec![]
            };
        matches!(
            name,
            "beta"
                | "bootstrap"
                | "build"
                | "cache"
                | "canister"
                | "deploy"
                | "deps"
                | "diagnose"
                | "fix"
                | "extension"
                | "generate"
                | "identity"
                | "info"
                | "_language-service"
                | "ledger"
                | "new"
                | "nns"
                | "ping"
                | "quickstart"
                | "remote"
                | "replica"
                | "schema"
                | "sns"
                | "start"
                | "stop"
                | "toolchain"
                | "upgrade"
                | "wallet"
        ) || extension_subcommands.contains(&name.to_owned())
    }
}

impl FromArgMatches for DfxCommand {
    fn from_arg_matches(matches: &ArgMatches) -> Result<Self, Error> {
        match matches.subcommand() {
            Some(("beta", args)) => Ok(Self::Beta(beta::BetaOpts::from_arg_matches(args)?)),
            Some(("bootstrap", args)) => Ok(Self::Bootstrap(
                bootstrap::BootstrapOpts::from_arg_matches(args)?,
            )),
            Some(("build", args)) => Ok(Self::Build(build::CanisterBuildOpts::from_arg_matches(
                args,
            )?)),
            Some(("cache", args)) => Ok(Self::Cache(cache::CacheOpts::from_arg_matches(args)?)),
            Some(("canister", args)) => Ok(Self::Canister(
                canister::CanisterOpts::from_arg_matches(args)?,
            )),
            Some(("deploy", args)) => Ok(Self::Deploy(deploy::DeployOpts::from_arg_matches(args)?)),
            Some(("deps", args)) => Ok(Self::Deps(deps::DepsOpts::from_arg_matches(args)?)),
            Some(("diagnose", args)) => Ok(Self::Diagnose(
                diagnose::DiagnoseOpts::from_arg_matches(args)?,
            )),
            Some(("fix", args)) => Ok(Self::Fix(fix::FixOpts::from_arg_matches(args)?)),
            // Some(("sns", args)) => Ok(Self::Sns(sns::SnsOpts::from_arg_matches(args)?)),
            Some(("extension", args)) => Ok(Self::Extension(
                extension::ExtensionOpts::from_arg_matches(args)?,
            )),
            Some(("generate", args)) => Ok(Self::Generate(
                generate::GenerateOpts::from_arg_matches(args)?,
            )),
            Some(("identity", args)) => Ok(Self::Identity(
                identity::IdentityOpts::from_arg_matches(args)?,
            )),
            Some(("info", args)) => Ok(Self::Info(info::InfoOpts::from_arg_matches(args)?)),
            Some(("_language-service", args)) => Ok(Self::LanguageServices(
                language_service::LanguageServiceOpts::from_arg_matches(args)?,
            )),
            Some(("ledger", args)) => Ok(Self::Ledger(ledger::LedgerOpts::from_arg_matches(args)?)),
            Some(("new", args)) => Ok(Self::New(new::NewOpts::from_arg_matches(args)?)),
            Some(("nns", args)) => Ok(Self::Nns(nns::NnsOpts::from_arg_matches(args)?)),
            Some(("ping", args)) => Ok(Self::Ping(ping::PingOpts::from_arg_matches(args)?)),
            Some(("quickstart", args)) => Ok(Self::Quickstart(
                quickstart::QuickstartOpts::from_arg_matches(args)?,
            )),
            Some(("remote", args)) => Ok(Self::Remote(remote::RemoteOpts::from_arg_matches(args)?)),
            Some(("replica", args)) => {
                Ok(Self::Replica(replica::ReplicaOpts::from_arg_matches(args)?))
            }
            Some(("schema", args)) => Ok(Self::Schema(schema::SchemaOpts::from_arg_matches(args)?)),
            Some(("sns", args)) => Ok(Self::Sns(sns::SnsOpts::from_arg_matches(args)?)),
            Some(("start", args)) => Ok(Self::Start(start::StartOpts::from_arg_matches(args)?)),
            Some(("stop", args)) => Ok(Self::Stop(stop::StopOpts::from_arg_matches(args)?)),
            Some(("toolchain", args)) => Ok(Self::Toolchain(
                toolchain::ToolchainOpts::from_arg_matches(args)?,
            )),
            Some(("upgrade", args)) => {
                Ok(Self::Upgrade(upgrade::UpgradeOpts::from_arg_matches(args)?))
            }
            Some(("wallet", args)) => Ok(Self::Wallet(wallet::WalletOpts::from_arg_matches(args)?)),
            Some((v, a)) => {
                let args = &std::env::args_os().collect::<Vec<_>>()[1..];
                let args = args.to_vec();
                return Ok(Self::ExtensionRun(args));

                if let Ok(ext_manager) = ExtensionManager::new(dfx_version(), false) {
                    if let Err(err) = ext_manager.run_extension(OsString::from(v), args) {
                        return Err(Error::raw(
                            ErrorKind::InvalidSubcommand,
                            format!("{}", err), // "The subcommand provided is not valid.",
                        ));
                    }
                };
                Err(Error::raw(
                    ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand,
                    "A subcommand is required.",
                ))
            }
            None => Err(Error::raw(
                ErrorKind::MissingSubcommand,
                "A subcommand is required.",
            )),
        }
    }

    fn update_from_arg_matches(&mut self, matches: &ArgMatches) -> Result<(), Error> {
        let cmd = Self::from_arg_matches(matches)?;
        *self = cmd;
        Ok(())
    }
}

/// sort subcommands alphabetically, help still gets printed as the last one
fn sort_clap_commands(cmd: &mut Command) {
    let mut cli_subcommands: Vec<String> = cmd
        .get_subcommands()
        .map(|v| v.get_display_name().unwrap_or_default().to_string())
        .collect();
    cli_subcommands.sort();
    let cli_subcommands: HashMap<String, usize> = cli_subcommands
        .into_iter()
        .enumerate()
        .map(|(i, v)| (v, i))
        .collect();
    for c in cmd.get_subcommands_mut() {
        let name = c.get_display_name().unwrap_or_default().to_string();
        let ord = *cli_subcommands.get(&name).unwrap_or(&999);
        *c = c.clone().display_order(ord);
    }
}
