use crate::lib::error::DfxResult;
use clap::{App, Arg, ArgMatches, SubCommand};

pub fn available() -> bool {
    true
}

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("canister").about("Generate a new canister in your project.")
        .arg(
            Arg::with_name("name")
                .help("The name of the canister to generate.")
                .required(true),
        )
}

pub fn exec(_args: &ArgMatches<'_>) -> DfxResult {
    println!("Canister generated.");

    Ok(())
}
