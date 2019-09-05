use crate::lib::error::{DfxResult, DfxError};
use clap::{App, Arg, ArgMatches, SubCommand};
use crate::config::dfinity::Config;
use crate::lib::api_client::{install_code, Client, ClientConfig};

pub fn available() -> bool {
    true
}

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("add")
        .about("Add a user to the key store.")
        .arg(
            Arg::with_name("canister")
                .help("The ID of the authentication to add.")
                .required(true),
        )
        .arg(
            Arg::with_name("wasm").help(
                "The wasm file to use. By default will use the wasm of the same canister name.",
            ).required(true),
        )
}

pub fn exec(args: &ArgMatches<'_>) -> DfxResult {
    // Read the config.
    let config = Config::from_current_dir()?;
    // get_path() returns the name of the config.
    let project_root = config.get_path().parent().unwrap();

    let wasm_path = args.value_of("wasm").unwrap();
    let canister_id = args.value_of("canister")?.parse::<u64>()?;

    let wasm = std::fs::read(wasm_path)?;

    let url = args.value_of("network")?;
    let client = Client::new(ClientConfig {
        url: url.to_string(),
    });
    install_code()

    Ok(())
}
