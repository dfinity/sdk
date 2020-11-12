use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use clap::{App, ArgMatches, Clap, FromArgMatches, IntoApp};

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

pub fn construct() -> App<'static> {
    IdentityOpt::into_app()
}

pub fn exec(env: &dyn Environment, args: &ArgMatches) -> DfxResult {
    let opts: IdentityOpt = IdentityOpt::from_arg_matches(args);
    match opts.subcmd {
        SubCommand::List(_v) => list::exec(env),
        SubCommand::New(v) => new::exec(env, v),
        SubCommand::GetPrincipal(_v) => principal::exec(env),
        SubCommand::Remove(v) => remove::exec(env, v),
        SubCommand::Rename(v) => rename::exec(env, v),
        SubCommand::Use(v) => r#use::exec(env, v),
        SubCommand::Whoami(_v) => whoami::exec(env),
    }
}
