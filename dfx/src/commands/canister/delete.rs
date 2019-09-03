use crate::lib::error::DfxResult;
use clap::{App, Arg, ArgMatches, SubCommand};

pub fn available() -> bool {
    true
}

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("delete")
        .about("Add a user to the key store.")
        .arg(
            Arg::with_name("name")
                .help("The name of the canister to delete.")
                .required(true),
        )
}

pub fn exec(_args: &ArgMatches<'_>) -> DfxResult {
    println!("Canister deleted.");

    Ok(())
}
