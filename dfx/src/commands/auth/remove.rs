use crate::lib::error::DfxResult;
use clap::{App, Arg, ArgMatches, SubCommand};

pub fn available() -> bool {
    true
}

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("remove")
        .about("Add a user to the key store.")
        .arg(Arg::with_name("name").help("The name of the authentication to add."))
}

pub fn exec(args: &ArgMatches<'_>) -> DfxResult {
    println!("Removed credentials for {}", args.value_of("name"));

    Ok(())
}
