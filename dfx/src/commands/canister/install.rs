use crate::lib::error::{DfxResult, DfxError};
use clap::{App, Arg, ArgMatches, SubCommand};
use crate::config::dfinity::Config;
use crate::lib::api_client::{install_code, Client, ClientConfig, CanisterInstallCodeCall, Blob, InstallResponseReply, Response};
use futures::future::{err, ok, Future};
use tokio::runtime::Runtime;
use std::path::PathBuf;

pub fn available() -> bool {
    true
}

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("install")
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

    let canister_id = args.value_of("canister").unwrap().parse::<u64>()?;
    let url = args.value_of("network").unwrap_or("http://localhost:8080");
    let wasm_path = args.value_of("wasm").unwrap();
    let wasm_path = PathBuf::from(project_root).join(wasm_path);

    let wasm = std::fs::read(wasm_path)?;
    let client = Client::new(ClientConfig {
        url: url.to_string(),
    });

    let install = install_code(client, CanisterInstallCodeCall {
        canister_id,
        module: Blob(wasm),
    }).and_then(|r| match r {
        Response::Accepted => {
            println!("Accepted");
            ok(())
        }
        Response::Replied {
            reply: InstallResponseReply { },
        } => {
            ok(())
        }
        Response::Rejected {
            reject_code,
            reject_message,
        } => err(DfxError::ClientError(reject_code, reject_message)),
        Response::Unknown => err(DfxError::Unknown("Unknown response".to_owned())),
    });

    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(install)
}
