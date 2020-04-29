#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use crate::commands::CliCommand;
use crate::lib::environment::{AgentEnvironment, Environment};
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use clap::{App, Arg, ArgMatches, SubCommand};

mod new;

fn builtins() -> Vec<CliCommand> {
    vec![CliCommand::new(new::construct(), new::exec)]
}

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("principal")
        .about(UserMessage::Principal.to_str())
        .subcommands(builtins().into_iter().map(|x| x.get_subcommand().clone()))
}

pub fn exec(env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult {
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
