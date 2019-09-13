use crate::lib::api_client::{install_code, Blob};
use crate::lib::env::{ClientEnv, ProjectConfigEnv};
use crate::lib::error::DfxResult;
use clap::{App, Arg, ArgMatches, SubCommand};
use std::path::PathBuf;
use tokio::runtime::Runtime;

fn is_number(v: String) -> Result<(), String> {
    v.parse::<u64>()
        .map_err(|_| String::from("The value must be a number."))
        .map(|_| ())
}

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("install")
        .about("Install a canister.")
        .arg(
            Arg::with_name("canister")
                .takes_value(true)
                .help("The canister ID (a number).")
                .required(true)
                .validator(is_number),
        )
        .arg(
            Arg::with_name("wasm")
                .help("The wasm file to use.")
                .required(true),
        )
}

pub fn exec<T>(env: &T, args: &ArgMatches<'_>) -> DfxResult
where
    T: ClientEnv + ProjectConfigEnv,
{
    // Read the config.
    let config = env.get_config().unwrap();
    let project_root = config.get_path().parent().unwrap();

    let canister_id = args.value_of("canister").unwrap().parse::<u64>()?;
    let wasm_path = args.value_of("wasm").unwrap();
    let wasm_path = PathBuf::from(project_root).join(wasm_path);

    let wasm = std::fs::read(wasm_path)?;
    let client = env.get_client();

    let install = install_code(client, canister_id, Blob(wasm), None);

    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(install)?;

    Ok(())
}
