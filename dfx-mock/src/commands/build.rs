use crate::commands::CliResult;
use crate::config::Config;
use clap::{Arg, ArgMatches, SubCommand, App};

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("build")
        .about("Build a cannister code, or all cannisters if no argument is passed.")
        .arg(
            Arg::with_name("cannister")
                .help("The cannister name to build.")
        )
}

pub fn exec(_args: &ArgMatches<'_>) -> CliResult {
    // Read the config.
    let _config = Config::from_current_dir()?;



    Ok(())
}
