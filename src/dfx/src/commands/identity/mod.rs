use crate::commands::CliCommand;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use clap::{App, ArgMatches, Clap, IntoApp};

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
pub struct IdentityOpt {}

pub fn construct() -> App<'static> {
    IdentityOpt::into_app().subcommands(builtins().into_iter().map(|x| x.get_subcommand().clone()))
}

fn builtins() -> Vec<CliCommand> {
    vec![
        CliCommand::new(list::construct(), list::exec),
        CliCommand::new(new::construct(), new::exec),
        CliCommand::new(remove::construct(), remove::exec),
        CliCommand::new(rename::construct(), rename::exec),
        CliCommand::new(r#use::construct(), r#use::exec),
        CliCommand::new(whoami::construct(), whoami::exec),
        CliCommand::new(principal::construct(), principal::exec),
    ]
}

pub fn exec(env: &dyn Environment, args: &ArgMatches) -> DfxResult {
    let subcommand = args.subcommand();

    if let Some((name, subcommand_args)) = subcommand {
        match builtins().into_iter().find(|x| name == x.get_name()) {
            Some(cmd) => cmd.execute(env, subcommand_args),
            None => Err(DfxError::UnknownCommand(format!(
                "Command {} not found.",
                name
            ))),
        }
    } else {
        construct().write_help(&mut std::io::stderr())?;
        eprintln!();
        eprintln!();
        Ok(())
    }
}
