use crate::{init_env, lib::error::DfxResult};

use clap::Parser;

use super::NetworkOpts;

mod deploy_wallet;
mod export;
mod get_wallet;
mod import;
mod list;
mod new;
mod principal;
mod remove;
mod rename;
mod set_wallet;
mod r#use;
mod whoami;

/// Manages identities used to communicate with the Internet Computer network.
/// Setting an identity enables you to test user-based access controls.
#[derive(Parser)]
#[clap(name("identity"))]
pub struct IdentityCommand {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    DeployWallet(NetworkOpts<deploy_wallet::DeployWalletOpts>),
    Export(NetworkOpts<export::ExportOpts>),
    GetWallet(NetworkOpts<get_wallet::GetWalletOpts>),
    Import(NetworkOpts<import::ImportOpts>),
    List(NetworkOpts<list::ListOpts>),
    New(NetworkOpts<new::NewIdentityOpts>),
    GetPrincipal(NetworkOpts<principal::GetPrincipalOpts>),
    Remove(NetworkOpts<remove::RemoveOpts>),
    Rename(NetworkOpts<rename::RenameOpts>),
    SetWallet(NetworkOpts<set_wallet::SetWalletOpts>),
    Use(NetworkOpts<r#use::UseOpts>),
    Whoami(NetworkOpts<whoami::WhoAmIOpts>),
}

pub fn dispatch(cmd: IdentityCommand) -> DfxResult {
    match cmd.subcmd {
        SubCommand::DeployWallet(v) => deploy_wallet::exec(
            &init_env(v.base_opts.env_opts)?,
            v.base_opts.command_opts,
            v.network,
        ),
        SubCommand::Export(v) => {
            export::exec(&init_env(v.base_opts.env_opts)?, v.base_opts.command_opts)
        }
        SubCommand::GetWallet(v) => get_wallet::exec(
            &init_env(v.base_opts.env_opts)?,
            v.base_opts.command_opts,
            v.network,
        ),
        SubCommand::List(v) => {
            list::exec(&init_env(v.base_opts.env_opts)?, v.base_opts.command_opts)
        }
        SubCommand::New(v) => new::exec(&init_env(v.base_opts.env_opts)?, v.base_opts.command_opts),
        SubCommand::GetPrincipal(v) => {
            principal::exec(&init_env(v.base_opts.env_opts)?, v.base_opts.command_opts)
        }
        SubCommand::Import(v) => {
            import::exec(&init_env(v.base_opts.env_opts)?, v.base_opts.command_opts)
        }
        SubCommand::Remove(v) => {
            remove::exec(&init_env(v.base_opts.env_opts)?, v.base_opts.command_opts)
        }
        SubCommand::Rename(v) => {
            rename::exec(&init_env(v.base_opts.env_opts)?, v.base_opts.command_opts)
        }
        SubCommand::SetWallet(v) => set_wallet::exec(
            &init_env(v.base_opts.env_opts)?,
            v.base_opts.command_opts,
            v.network,
        ),
        SubCommand::Use(v) => {
            r#use::exec(&init_env(v.base_opts.env_opts)?, v.base_opts.command_opts)
        }
        SubCommand::Whoami(v) => {
            whoami::exec(&init_env(v.base_opts.env_opts)?, v.base_opts.command_opts)
        }
    }
}
