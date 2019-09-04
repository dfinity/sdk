use crate::lib::error::DfxResult;
use clap::{App, Arg, ArgMatches, SubCommand};

pub fn available() -> bool {
    true
}

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("create")
        .about("Create a new canister.")
        .arg(
            Arg::with_name("name")
                .help("The name of the canister to create.")
                .required(true),
        )
}

pub fn exec(_args: &ArgMatches<'_>) -> DfxResult {
    println!("Canister created.");

    Ok(())
}
