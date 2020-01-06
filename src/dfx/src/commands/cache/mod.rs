use crate::commands::CliCommand;
use crate::lib::env::{BinaryCacheEnv, VersionEnv};
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use clap::{App, ArgMatches, SubCommand};

mod delete;
mod install;
mod list;
mod show;

fn builtins<T>() -> Vec<CliCommand<T>>
where
    T: BinaryCacheEnv + VersionEnv,
{
    vec![
        CliCommand::new(delete::construct(), delete::exec),
        CliCommand::new(list::construct(), list::exec),
        CliCommand::new(install::construct(), install::exec),
        CliCommand::new(show::construct(), show::exec),
    ]
}

pub fn construct<T>() -> App<'static, 'static>
where
    T: BinaryCacheEnv + VersionEnv,
{
    SubCommand::with_name("cache")
        .about(UserMessage::ManageCache.to_str())
        .subcommands(
            builtins::<T>()
                .into_iter()
                .map(|x| x.get_subcommand().clone()),
        )
}

pub fn exec<T>(env: &T, args: &ArgMatches<'_>) -> DfxResult
where
    T: BinaryCacheEnv + VersionEnv,
{
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
        construct::<T>().write_help(&mut std::io::stderr())?;
        eprintln!();
        eprintln!();
        Ok(())
    }
}
