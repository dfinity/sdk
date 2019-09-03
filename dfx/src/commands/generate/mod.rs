use crate::commands::{add_builtin, CliCommand};
use crate::config::dfinity::Config;
use crate::lib::error::{DfxError, DfxResult};
use clap::{App, ArgMatches, SubCommand};

mod canister;
mod upgrade;

pub fn available() -> bool {
    Config::from_current_dir().is_ok()
}

pub fn construct() -> App<'static, 'static> {
    // There is a difference in arguments between in-project versus global.
    let mut app =
        SubCommand::with_name("generate").about("Generate or transform files in your project.");

    app = app.subcommands(builtins().into_iter().map(|x| x.get_subcommand().clone()));

    app
}

pub fn builtins() -> Vec<CliCommand> {
    let mut v: Vec<CliCommand> = Vec::new();

    add_builtin(
        &mut v,
        canister::available(),
        canister::construct(),
        canister::exec,
    );
    add_builtin(
        &mut v,
        upgrade::available(),
        upgrade::construct(),
        upgrade::exec,
    );

    v
}

pub fn exec(args: &ArgMatches<'_>) -> DfxResult {
    let subcommand = args.subcommand();

    if let (name, Some(subcommand_args)) = subcommand {
        match builtins().into_iter().find(|x| name == x.get_name()) {
            Some(cmd) => cmd.execute(subcommand_args),
            None => Err(DfxError::UnknownCommand(format!(
                "Command {} not found.",
                name
            ))),
        }
    } else {
        construct().write_help(&mut std::io::stderr())?;
        println!();
        println!();
        Ok(())
    }
}
