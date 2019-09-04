use crate::commands::{add_builtin, CliCommand};
use crate::config::dfinity::Config;
use crate::lib::error::{DfxError, DfxResult};
use clap::{App, Arg, ArgMatches, SubCommand};

mod create;
mod delete;
mod install;
mod upgrade;

pub fn available() -> bool {
    true
}

pub fn construct() -> App<'static, 'static> {
    // There is a difference in arguments between in-project versus global.
    let mut app =
        SubCommand::with_name("canister").about("Manage canisters on the global network.");

    if Config::from_current_dir().is_err() {
        app = app.arg(
            Arg::with_name("network")
                .help(r#"The network URL to use. Either "main", "local" or a URL."#)
                .required(true)
                .validator(|value| match value.as_str() {
                    "main" => Ok(()),
                    "local" => Ok(()),
                    x => reqwest::Url::parse(x)
                        .map_err(|_| r#""main", "local" or URL expected."#.to_owned())
                        .map(|_| ()),
                }),
        );
    }

    app = app.subcommands(builtins().into_iter().map(|x| x.get_subcommand().clone()));

    app
}

pub fn builtins() -> Vec<CliCommand> {
    let mut v: Vec<CliCommand> = Vec::new();

    add_builtin(
        &mut v,
        create::available(),
        create::construct(),
        create::exec,
    );
    add_builtin(
        &mut v,
        delete::available(),
        delete::construct(),
        delete::exec,
    );
    add_builtin(
        &mut v,
        install::available(),
        install::construct(),
        install::exec,
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
