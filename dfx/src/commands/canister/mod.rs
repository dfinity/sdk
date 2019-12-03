use crate::commands::CliCommand;
use crate::lib::env::{BinaryResolverEnv, ClientEnv, ProjectConfigEnv};
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use clap::{App, ArgMatches, SubCommand};

mod call;
mod install;
mod request_status;

fn builtins<T>() -> Vec<CliCommand<T>>
where
    T: ClientEnv + ProjectConfigEnv + BinaryResolverEnv,
{
    vec![
        CliCommand::new(call::construct(), call::exec),
        CliCommand::new(install::construct(), install::exec),
        CliCommand::new(request_status::construct(), request_status::exec),
    ]
}

pub fn construct<T>() -> App<'static, 'static>
where
    T: ClientEnv + ProjectConfigEnv + BinaryResolverEnv,
{
    SubCommand::with_name("canister")
        .about(UserMessage::ManageCanister.to_str())
        .subcommands(
            builtins::<T>()
                .into_iter()
                .map(|x| x.get_subcommand().clone()),
        )
}

pub fn exec<T>(env: &T, args: &ArgMatches<'_>) -> DfxResult
where
    T: ClientEnv + ProjectConfigEnv + BinaryResolverEnv,
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
