use crate::commands::CliResult;
use crate::config::dfinity::Config;
use clap::{App, Arg, ArgMatches, SubCommand};

pub fn available() -> bool {
    Config::from_current_dir().is_ok()
}

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("send")
        .about("Send a message to a canister, and potentially wait for the answer.")
        .arg(Arg::with_name("canister").help("The canister name to send to."))
}

pub fn exec(_args: &ArgMatches<'_>) -> CliResult {
    // Read the config.
    let _config = Config::from_current_dir()?;

    Ok(())
}
