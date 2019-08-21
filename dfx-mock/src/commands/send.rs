use crate::commands::CliResult;
use crate::config::Config;
use clap::{Arg, ArgMatches, SubCommand, App};

pub fn available() -> bool {
    Config::from_current_dir().is_ok()
}

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("send")
        .about("Send a message to a cannister, and potentially wait for the answer.")
        .arg(
            Arg::with_name("cannister")
                .help("The cannister name to send to.")
        )
}

pub fn exec(_args: &ArgMatches<'_>) -> CliResult {
    // Read the config.
    let _config = Config::from_current_dir()?;

    Ok(())
}
