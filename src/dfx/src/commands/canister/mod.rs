use crate::commands::CliCommand;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::provider::create_agent_environment;

use anyhow::bail;
use clap::{App, ArgMatches, Clap, FromArgMatches, IntoApp};

mod call;
mod create;
mod delete;
mod id;
mod install;
mod request_status;
mod set_controller;
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
        CliCommand::new(set_controller::construct(), set_controller::exec),
        CliCommand::new(start::construct(), start::exec),
        CliCommand::new(status::construct(), status::exec),
        CliCommand::new(stop::construct(), stop::exec),
    ]
}

/// Manages canisters deployed on a network replica.
#[derive(Clap)]
#[clap(name("canister"))]
pub struct CanisterOpts {
    // Override the compute network to connect to. By default, the local network is used.
    #[clap(long)]
    network: Option<String>,
}

pub fn construct() -> App<'static> {
    CanisterOpts::into_app().subcommands(builtins().into_iter().map(|x| x.get_subcommand().clone()))
}

pub fn exec(env: &dyn Environment, args: &ArgMatches) -> DfxResult {
    let opts: CanisterOpts = CanisterOpts::from_arg_matches(args);
    let subcommand = args.subcommand();
    let agent_env = create_agent_environment(env, opts.network)?;

    if let Some((name, subcommand_args)) = subcommand {
        match builtins().into_iter().find(|x| name == x.get_name()) {
            Some(cmd) => cmd.execute(&agent_env, subcommand_args),
            None => bail!("Command {} not found.", name),
        }
    } else {
        construct().write_help(&mut std::io::stderr())?;
        eprintln!();
        eprintln!();
        Ok(())
    }
}
