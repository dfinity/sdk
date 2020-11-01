use crate::commands::CliCommand;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use clap::{App, ArgMatches, Clap, FromArgMatches, IntoApp};

mod list;
mod new;
mod principal;
mod remove;
mod rename;
mod r#use;
mod whoami;

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

pub fn construct() -> App<'static> {
    App::new("identity")
        .about(UserMessage::ManageIdentity.to_str())
        .subcommands(builtins().into_iter().map(|x| x.get_subcommand().clone()))
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
