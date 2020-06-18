use crate::commands::CliCommand;
use crate::lib::environment::{AgentEnvironment, Environment};
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use clap::{App, Arg, ArgMatches, SubCommand};

mod call;
mod create;
mod id;
mod install;
mod query;
mod request_status;

fn builtins() -> Vec<CliCommand> {
    vec![
        CliCommand::new(call::construct(), call::exec),
        CliCommand::new(create::construct(), create::exec),
        CliCommand::new(id::construct(), id::exec),
        CliCommand::new(install::construct(), install::exec),
        CliCommand::new(query::construct(), query::exec),
        CliCommand::new(request_status::construct(), request_status::exec),
    ]
}

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("canister")
        .about(UserMessage::ManageCanister.to_str())
        .arg(
            Arg::with_name("provider")
                .help(UserMessage::CanisterComputeProvider.to_str())
                .long("provider")
                .validator(|v| {
                    reqwest::Url::parse(&v)
                        .map(|_| ())
                        .map_err(|_| "should be a valid URL.".to_string())
                })
                .takes_value(true),
        )
        .subcommands(builtins().into_iter().map(|x| x.get_subcommand().clone()))
}

pub fn exec(env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult {
    let subcommand = args.subcommand();

    // Need storage for AgentEnvironment ownership.
    let mut _agent_env: Option<AgentEnvironment<'_>> = None;
    let env = if args.is_present("provider") {
        _agent_env = Some(AgentEnvironment::new(
            env,
            args.value_of("provider").expect("Could not find provider."),
        ));
        _agent_env.as_ref().unwrap()
    } else {
        env
    };

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
