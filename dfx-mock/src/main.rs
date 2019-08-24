use clap::{App, AppSettings};
use crate::commands::CliError;
use crate::config::DFX_VERSION;

mod commands;
mod config;
mod util;

fn cli() -> App<'static, 'static> {
    App::new("dfx")
        .about("The DFINITY Executor.")
        .version(DFX_VERSION)
        .setting(AppSettings::ColoredHelp)
        .subcommands(
            commands::builtin().into_iter().map(|x| x.get_subcommand().clone())
        )
}

fn exec(args: &clap::ArgMatches<'_>) -> commands::CliResult {
    let (name, subcommand_args) = match args.subcommand() {
        (name, Some(args)) => (name, args),
        _ => {
            cli().write_help(&mut std::io::stderr())?;
            println!();
            println!();
            return Ok(());
        }
    };

    match commands::builtin().into_iter().find(|x| name == x.get_name()) {
        Some(cmd) => cmd.execute(subcommand_args),
        _ => {
            cli().write_help(&mut std::io::stderr())?;
            println!();
            println!();
            Err(CliError::new(failure::format_err!("Command {} unknown.", name), 101))
        }
    }
}

fn main() {
    let matches = cli().get_matches();

    match exec(&matches) {
        Ok(()) => ::std::process::exit(0),
        Err(err) => {
            let CliError{ error, exit_code } = err;
            if let Some(err) = error {
                println!("An error occured:");
                println!("{}", err);
            }
            ::std::process::exit(exit_code)
        }
    }
}
