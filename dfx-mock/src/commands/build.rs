use crate::commands::CliResult;
use crate::config::Config;
use clap::{Arg, ArgMatches, SubCommand, App};
use crate::util::FakeProgress;

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
    let _config = Config::from_current_dir()?;

    let mut fp = FakeProgress::new();
    fp.add(
        1000..2000,
        |bar| {

        },
        |bar| {},
    );

    Ok(())
}
