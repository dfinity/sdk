use crate::lib::error::DfxResult;
use clap::{App, Arg, ArgMatches, SubCommand};

pub fn available() -> bool {
    true
}

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("add")
        .about("Add a user to the key store.")
        .arg(
            Arg::with_name("name")
                .help("The name of the authentication to add.")
                .required(true),
        )
        .arg(
            Arg::with_name("wasm")
                .help("The wasm file to use. By default will use the wasm of the same canister name.")
        )
}

pub fn exec(_args: &ArgMatches<'_>) -> DfxResult {
    println!("Installed wasm");

    Ok(())
}
