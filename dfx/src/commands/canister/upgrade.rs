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
                .help("The name of the canister to delete.")
                .required(true),
        )
        .arg(
            Arg::with_name("from")
                .help("Upgrade a canister from this version. By default will use the current version from the network.")
        )
        .arg(
            Arg::with_name("to")
                .help("Upgrade a canister to this version. By default uses the version from the project.")
        )
}

pub fn exec(_args: &ArgMatches<'_>) -> DfxResult {
    println!("Upgraded canister successfully");

    Ok(())
}
