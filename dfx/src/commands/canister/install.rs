use crate::config::dfinity::Config;
use crate::lib::api_client::{install_code, Blob, CanisterInstallCodeCall, Client, ClientConfig};
use crate::lib::error::DfxResult;
use clap::{App, Arg, ArgMatches, SubCommand};
use std::path::PathBuf;
use tokio::runtime::Runtime;

pub fn available() -> bool {
    true
}

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("install")
        .about("Install a canister on the client.")
        .arg(
            Arg::with_name("canister")
                .help("The ID of the authentication to add.")
                .required(true),
        )
        .arg(
            Arg::with_name("wasm")
                .help(
                    "The wasm file to use. By default will use the wasm of the same canister name.",
                )
                .required(true),
        )
}

pub fn exec(args: &ArgMatches<'_>) -> DfxResult {
    // Read the config.
    let config = Config::from_current_dir()?;
    // get_path() returns the name of the config.
    let project_root = config.get_path().parent().unwrap();

    let canister_id = args.value_of("canister").unwrap().parse::<u64>()?;
    let url = args.value_of("network").unwrap_or("http://localhost:8080");
    let wasm_path = args.value_of("wasm").unwrap();
    let wasm_path = PathBuf::from(project_root).join(wasm_path);

    let wasm = std::fs::read(wasm_path)?;
    let client = Client::new(ClientConfig {
        url: url.to_string(),
    });

    let install = install_code(
        client,
        CanisterInstallCodeCall {
            canister_id,
            module: Blob(wasm),
        },
    );

    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(install)
}
