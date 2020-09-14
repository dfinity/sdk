use crate::commands::CliCommand;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::lib::provider::create_agent_environment;
use clap::{App, Arg, ArgMatches, SubCommand};
use humanize_rs::duration::parse;

mod call;
mod create;
mod delete;
mod id;
mod install;
mod request_status;
mod start;
mod status;
mod stop;

fn builtins() -> Vec<CliCommand> {
    vec![
        CliCommand::new(call::construct(), call::exec),
        CliCommand::new(create::construct(), create::exec),
        CliCommand::new(delete::construct(), delete::exec),
        CliCommand::new(id::construct(), id::exec),
        CliCommand::new(install::construct(), install::exec),
        CliCommand::new(request_status::construct(), request_status::exec),
        CliCommand::new(start::construct(), start::exec),
        CliCommand::new(status::construct(), status::exec),
        CliCommand::new(stop::construct(), stop::exec),
    ]
}

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("canister")
        .about(UserMessage::ManageCanister.to_str())
        .arg(
            Arg::with_name("network")
                .help(UserMessage::CanisterComputeNetwork.to_str())
                .long("network")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("expiry-duration")
                .long("expiry-duration")
                .help(UserMessage::CanisterCallExpiryDuration.to_str())
                .takes_value(true)
                .default_value("5m")
                .validator(expiry_duration_validator),
        )
        .subcommands(builtins().into_iter().map(|x| x.get_subcommand().clone()))
}

fn expiry_duration_validator(expiry_duration: String) -> Result<(), String> {
    if let Ok(_ed) = parse(&expiry_duration) {
        return Ok(());
    }
    let err = format!(
        r#""Invalid input: {}. Expected a duration-type string e.g. "1h", "1h 30m""#,
        expiry_duration
    );
    Err(err)
}

pub fn exec(env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult {
    let subcommand = args.subcommand();

    let agent_env = create_agent_environment(env, args)?;

    if let (name, Some(subcommand_args)) = subcommand {
        match builtins().into_iter().find(|x| name == x.get_name()) {
            Some(cmd) => cmd.execute(&agent_env, subcommand_args),
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
