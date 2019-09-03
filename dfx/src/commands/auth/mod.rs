use crate::config::dfinity::Config;
use crate::lib::error::{DfxError, DfxResult};
use clap::{App, Arg, ArgMatches, SubCommand};
use crate::commands::{CliCommand, add_builtin};

mod add;
mod list;
mod remove;

pub fn available() -> bool { true }

pub fn construct() -> App<'static, 'static> {
    // There is a difference in arguments between in-project versus global.
    let mut app = SubCommand::with_name("auth")
        .about("Manage authentications and credentials.");

    if Config::from_current_dir().is_err() {
        app = app.arg(
            Arg::with_name("net")
                .help("The network URL to use. By default will use").required(true),
        );
    }

    app = app.subcommands(builtins().into_iter().map(|x| x.get_subcommand().clone()));

    app
}

pub fn builtins() -> Vec<CliCommand> {
    let mut v: Vec<CliCommand> = Vec::new();

    add_builtin(&mut v, add::available(), add::construct(), add::exec);
    add_builtin(&mut v, list::available(), list::construct(), list::exec);
    add_builtin(&mut v, remove::available(), remove::construct(), remove::exec);

    v
}

pub fn exec(args: &ArgMatches<'_>) -> DfxResult {
    let subcommand = args.subcommand();

    if let (name, Some(subcommand_args)) = subcommand {
        match builtins().into_iter().find(|x| name == x.get_name()) {
            Some(cmd) => cmd.execute(subcommand_args),
            None => Err(DfxError::UnknownCommand(format!("Command {} not found.", name))),
        }
    } else {
        construct().write_help(&mut std::io::stderr())?;
        println!();
        println!();
        Ok(())
    }
}
