use crate::commands::CliCommand;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use anyhow::anyhow;
use clap::{App, ArgMatches, Clap, IntoApp};

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

/// Manages the dfx version cache.
#[derive(Clap)]
#[clap(name("cache"))]
pub struct CacheOpts {}

pub fn construct() -> App<'static> {
    CacheOpts::into_app().subcommands(builtins().into_iter().map(|x| x.get_subcommand().clone()))
}

pub fn exec(env: &dyn Environment, args: &ArgMatches) -> DfxResult {
    let subcommand = args.subcommand();

    if let Some((name, subcommand_args)) = subcommand {
        match builtins().into_iter().find(|x| name == x.get_name()) {
            Some(cmd) => cmd.execute(env, subcommand_args),
            None => Err(anyhow!("Command '{}' not found.", name)),
        }
    } else {
        construct().write_help(&mut std::io::stderr())?;
        eprintln!();
        eprintln!();
        Ok(())
    }
}
