use clap::{App, AppSettings};

mod commands;
mod config;
mod lib;
mod util;

use crate::commands::CliCommand;
use crate::lib::env::{BinaryCacheEnv, InProjectEnvironment, VersionEnv};
use crate::lib::error::*;

fn cli<T>(env: &T) -> App<'_, '_>
where
    T: VersionEnv,
{
    App::new("dfx")
        .about("The DFINITY Executor.")
        .version(env.get_version().as_str())
        .global_setting(AppSettings::ColoredHelp)
        .subcommands(
            commands::builtin()
                .into_iter()
                .map(|x: CliCommand<InProjectEnvironment>| x.get_subcommand().clone()),
        )
}

fn exec(env: &InProjectEnvironment, args: &clap::ArgMatches<'_>, cli: &App<'_, '_>) -> DfxResult {
    let (name, subcommand_args) = match args.subcommand() {
        (name, Some(args)) => (name, args),
        _ => {
            cli.write_help(&mut std::io::stderr())?;
            println!();
            println!();
            return Ok(());
        }
    };

    match commands::builtin()
        .into_iter()
        .find(|x| name == x.get_name())
    {
        Some(cmd) => cmd.execute(env, subcommand_args),
        _ => {
            cli.write_help(&mut std::io::stderr())?;
            println!();
            println!();
            Err(DfxError::UnknownCommand(name.to_owned()))
        }
    }
}

fn main() {
    // Build the environment.
    let env = InProjectEnvironment::from_current_dir().unwrap();

    let matches = cli(&env).get_matches();

    if !env.is_installed().unwrap() {
        env.install().unwrap();
        println!("Version v{} installed successfully.", env.get_version());
    } else {
        println!("Version v{} already installed.", env.get_version());
    }

    if let Err(err) = exec(&env, &matches, &(cli(&env))) {
        println!("An error occured:\n{:#?}", err);
        ::std::process::exit(255)
    }
}
