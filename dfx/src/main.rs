use clap::{App, AppSettings, Arg};
use crate::lib::api_client::{Client, ClientConfig};
use crate::lib::env::{Env};

mod commands;
pub mod lib;

use lib::error::*;

const HOST_FLAG: &str = "host";
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn cli() -> App<'static, 'static> {
    App::new("dfx")
        .about("The DFINITY Executor.")
        .version(VERSION)
        .setting(AppSettings::ColoredHelp)
        .arg(
            Arg::with_name(HOST_FLAG)
                .global(true)
                .help("The host (with port) to send the query to.")
                .long(HOST_FLAG)
                .takes_value(true)
        )
        .subcommands(
            commands::ctors(),
        )
}

fn exec(env: &'static Env, args: &clap::ArgMatches<'_>) -> DfxResult {
    let (name, subcommand_args) = match args.subcommand() {
        (name, Some(args)) => (name, args),
        _ => {
            cli().write_help(&mut std::io::stderr())?;
            println!();
            println!();
            return Ok(());
        }
    };

    match commands::builtin(&env)
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
    let host = matches.value_of(HOST_FLAG);

    let default_config = ClientConfig::default();
    let default_host = default_config.host;

    let env = Env {
        client: Client::new(ClientConfig {
            host: host.map_or_else(|| default_host, |x| x.to_string()),
            .. default_config
        })
    };

    match exec(Box::leak(Box::new(env)), &matches) {
        Ok(()) => ::std::process::exit(0),
        Err(err) => {
            println!("An error occured:\n{:#?}", err);
            ::std::process::exit(255)
        }
    }
}
