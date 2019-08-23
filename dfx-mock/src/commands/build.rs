use crate::commands::CliResult;
use crate::config::dfinity::Config;
use clap::{Arg, ArgMatches, SubCommand, App};
use crate::config::cache::binary_command;

pub fn available() -> bool {
    Config::from_current_dir().is_ok()
}

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("build")
        .about("Build a canister code, or all canisters if no argument is passed.")
        .arg(
            Arg::with_name("canister")
                .help("The canister name to build.")
        )
}

pub fn exec(_args: &ArgMatches<'_>) -> CliResult {
    // Read the config.
    let config = Config::from_current_dir()?;
    // get_path() returns the name of the config.
    let project_root = config.get_path().parent().unwrap();

    binary_command(&config, "asc")?
        .arg(project_root.join("app/canisters/hello/main.as").into_os_string())
        .output()?;

    Ok(())
}
