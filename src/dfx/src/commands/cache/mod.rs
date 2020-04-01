use crate::commands::CliCommand;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use clap::{App, ArgMatches, SubCommand};

mod delete;
mod install;
mod list;
mod show;

fn builtins() -> Vec<CliCommand> {
    vec![
        CliCommand::new(delete::construct(), delete::exec),
        CliCommand::new(list::construct(), list::exec),
        CliCommand::new(install::construct(), install::exec),
        CliCommand::new(show::construct(), show::exec),
    ]
}

pub fn construct() -> App<'static> {
    SubCommand::with_name("cache")
        .about(UserMessage::ManageCache.to_str())
        .subcommands(builtins().into_iter().map(|x| x.get_subcommand().clone()))
}

pub fn exec(env: &dyn Environment, args: &ArgMatches) -> DfxResult {
    let subcommand = args.subcommand();

    if let (name, Some(subcommand_args)) = subcommand {
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
