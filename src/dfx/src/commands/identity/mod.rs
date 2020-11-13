use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use clap::Clap;

mod list;
mod new;
mod principal;
mod remove;
mod rename;
mod r#use;
mod whoami;

/// Manages identities used to communicate with the Internet Computer network.
/// Setting an identity enables you to test user-based access controls.
#[derive(Clap)]
#[clap(name("identity"))]
pub struct IdentityOpt {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    List(list::ListOpts),
    New(new::NewIdentityOpts),
    GetPrincipal(principal::GetPrincipalOpts),
    Remove(remove::RemoveOpts),
    Rename(rename::RenameOpts),
    Use(r#use::UseOpts),
    Whoami(whoami::WhoAmIOpts),
}

pub fn exec(env: &dyn Environment, opts: IdentityOpt) -> DfxResult {
    match opts.subcmd {
        SubCommand::List(v) => list::exec(env, v),
        SubCommand::New(v) => new::exec(env, v),
        SubCommand::GetPrincipal(v) => principal::exec(env, v),
        SubCommand::Remove(v) => remove::exec(env, v),
        SubCommand::Rename(v) => rename::exec(env, v),
        SubCommand::Use(v) => r#use::exec(env, v),
        SubCommand::Whoami(v) => whoami::exec(env, v),
    }
}
