use crate::lib::error::DfxResult;
use clap::{App, Arg, ArgMatches, SubCommand};

pub fn available() -> bool {
    true
}

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("remove")
        .about("Add a user to the key store.")
        .arg(
            Arg::with_name("name")
                .help("The name of the upgrade migration step.")
                .required(true),
        )
        .arg(Arg::with_name("from").help("Which version this upgrade should be applied from."))
        .arg(Arg::with_name("to").help("Which version should this upgrades to."))
}

pub fn exec(_args: &ArgMatches<'_>) -> DfxResult {
    println!("Upgrade generated...");

    Ok(())
}
