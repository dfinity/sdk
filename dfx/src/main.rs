use clap::{App, AppSettings};

mod commands;
mod config;
mod lib;
mod util;

use crate::commands::CliCommand;
use crate::config::dfinity::Config;
use crate::lib::env::{
    BinaryCacheEnv, BinaryResolverEnv, ClientEnv, GlobalEnvironment, InProjectEnvironment,
    PlatformEnv, ProjectConfigEnv, VersionEnv,
};
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

fn exec<T>(env: &T, args: &clap::ArgMatches<'_>, cli: &App<'_, '_>) -> DfxResult
where
    T: BinaryCacheEnv + VersionEnv + BinaryResolverEnv + ClientEnv + PlatformEnv + ProjectConfigEnv,
{
    let (name, subcommand_args) = match args.subcommand() {
        (name, Some(args)) => (name, args),
        _ => {
            cli.write_help(&mut std::io::stderr())?;
            eprintln!();
            eprintln!();
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
            eprintln!();
            eprintln!();
            Err(DfxError::UnknownCommand(name.to_owned()))
        }
    }
}

fn main() {
    let result = {
        if Config::from_current_dir().is_ok() {
            // Build the environment.
            let env = InProjectEnvironment::from_current_dir().unwrap();
            let matches = cli(&env).get_matches();

            exec(&env, &matches, &(cli(&env)))
        } else {
            let env = GlobalEnvironment::from_current_dir().unwrap();
            let matches = cli(&env).get_matches();

            exec(&env, &matches, &(cli(&env)))
        }
    };

    match result {
        Ok(()) => {}
        Err(DfxError::BuildError(err)) => {
            eprintln!("Build failed. Reason:");
            eprintln!("  {}", err);
            std::process::exit(255)
        }
        Err(err) => {
            eprintln!("An error occured:\n{:#?}", err);
            std::process::exit(255)
        }
    }
}
