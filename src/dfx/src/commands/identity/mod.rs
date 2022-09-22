use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::NetworkOpt;

use clap::Parser;

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
pub struct IdentityOpts {
    #[clap(flatten)]
    network: NetworkOpt,

    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    DeployWallet(deploy_wallet::DeployWalletOpts),
    Export(export::ExportOpts),
    GetWallet(get_wallet::GetWalletOpts),
    Import(import::ImportOpts),
    List(list::ListOpts),
    New(new::NewIdentityOpts),
    GetPrincipal(principal::GetPrincipalOpts),
    Remove(remove::RemoveOpts),
    Rename(rename::RenameOpts),
    SetWallet(set_wallet::SetWalletOpts),
    Use(r#use::UseOpts),
    Whoami(whoami::WhoAmIOpts),
}

pub fn exec(env: &dyn Environment, opts: IdentityOpts) -> DfxResult {
    match opts.subcmd {
        SubCommand::DeployWallet(v) => deploy_wallet::exec(env, v, opts.network.network),
        SubCommand::Export(v) => export::exec(env, v),
        SubCommand::GetWallet(v) => get_wallet::exec(env, v, opts.network.network),
        SubCommand::List(v) => list::exec(env, v),
        SubCommand::New(v) => new::exec(env, v),
        SubCommand::GetPrincipal(v) => principal::exec(env, v),
        SubCommand::Import(v) => import::exec(env, v),
        SubCommand::Remove(v) => remove::exec(env, v),
        SubCommand::Rename(v) => rename::exec(env, v),
        SubCommand::SetWallet(v) => set_wallet::exec(env, v, opts.network.network),
        SubCommand::Use(v) => r#use::exec(env, v),
        SubCommand::Whoami(v) => whoami::exec(env, v),
    }
}
