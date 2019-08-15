extern crate clap;
extern crate serde_json;

use clap::App;
use crate::commands::CliError;

mod commands;
mod config;
mod util;

fn cli() -> App<'static, 'static> {
    App::new("dfx")
        .about("DFINITY Executor")
        .version("v0.1.0")
        .subcommands(
            commands::builtin().into_iter().map(|x| x.get_subcommand().clone())
        )
}

fn exec(args: &clap::ArgMatches<'_>) -> commands::CliResult {
    let (name, subcommand_args) = match args.subcommand() {
        (name, Some(args)) => (name, args),
        _ => {
            cli().print_help()?;
            return Ok(());
        }
    };

    if let Some(cmd) = commands::builtin().into_iter().find(|x| name == x.get_name()) {
        cmd.execute(subcommand_args)
    } else {
        Err(CliError::new(failure::format_err!("Command {} unknown.", name), 101))
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
