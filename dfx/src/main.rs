use clap::{App, AppSettings};

mod commands;
pub mod lib;

use lib::error::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn cli() -> App<'static, 'static> {
    App::new("dfx")
        .about("The DFINITY Executor.")
        .version(VERSION)
        .setting(AppSettings::ColoredHelp)
        .subcommands(
            commands::builtin()
                .into_iter()
                .map(|x| x.get_subcommand().clone()),
        )
}

fn exec(args: &clap::ArgMatches<'_>) -> DfxResult {
    let (name, subcommand_args) = match args.subcommand() {
        (name, Some(args)) => (name, args),
        _ => {
            cli().write_help(&mut std::io::stderr())?;
            println!();
            println!();
            return Ok(());
        }
    };

    match commands::builtin()
        .into_iter()
        .find(|x| name == x.get_name())
    {
        Some(cmd) => cmd.execute(subcommand_args),
        _ => {
            cli().write_help(&mut std::io::stderr())?;
            println!();
            println!();
            Err(DfxError::UnknownCommand(format!(
                "Command {} unknown.",
                name
            )))
        }
    }
}

fn main() {
    let matches = cli().get_matches();

    match exec(&matches) {
        Ok(()) => ::std::process::exit(0),
        Err(err) => {
            println!("An error occured:\n{:#?}", err);
            ::std::process::exit(255)
        }
    }
}
