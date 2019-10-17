use crate::lib::api_client::{install_code, request_status, QueryResponseReply, ReadResponse};
use crate::lib::env::{ClientEnv, ProjectConfigEnv};
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::util::clap::validators;
use crate::util::print_idl_blob;
use clap::{App, Arg, ArgMatches, SubCommand};
use ic_http_agent::{Blob, CanisterId};
use std::path::PathBuf;
use tokio::runtime::Runtime;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("install")
        .about(UserMessage::InstallCanister.to_str()))
        .arg(
            Arg::with_name("canister")
                .takes_value(true)
                .help(UserMessage::CanisterId.to_str()))
                .required(true)
                .validator(validators::is_canister_id),
        )
        .arg(
            Arg::with_name("wait")
                .help(UserMessage::WaitForResult.to_str()))
                .long("wait")
                .short("w")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("wasm")
                .help(UserMessage::WasmFile.to_str()))
                .required(true),
        )
}

pub fn exec<T>(env: &T, args: &ArgMatches<'_>) -> DfxResult
where
    T: ClientEnv + ProjectConfigEnv,
{
    // Read the config.
    let config = env
        .get_config()
        .ok_or_else(DfxError::CommandMustBeRunInAProject)?;

    let project_root = config.get_path().parent().unwrap();

    let canister_id = args.value_of("canister").unwrap().parse::<CanisterId>()?;
    let wasm_path = args.value_of("wasm").unwrap();
    let wasm_path = PathBuf::from(project_root).join(wasm_path);

    let wasm = std::fs::read(wasm_path)?;
    let client = env.get_client();

    let install = install_code(client, canister_id, Blob(wasm), None);

    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    let request_id = runtime.block_on(install)?;

    if args.is_present("wait") {
        let request_status = request_status(env.get_client(), request_id);
        let mut runtime = Runtime::new().expect("Unable to create a runtime");
        match runtime.block_on(request_status) {
            Ok(ReadResponse::Pending) => {
                eprintln!("Pending");
                println!("0x{}", String::from(request_id));
                Ok(())
            }
            Ok(ReadResponse::Replied { reply }) => {
                if let Some(QueryResponseReply { arg: blob }) = reply {
                    print_idl_blob(&blob)?;
                }
                Ok(())
            }
            Ok(ReadResponse::Rejected {
                reject_code,
                reject_message,
            }) => Err(DfxError::ClientError(reject_code, reject_message)),
            // TODO(SDK-446): remove this matcher when moving api_client to ic_http_agent.
            // `install` cannot return Unknown.
            Ok(ReadResponse::Unknown) => Err(DfxError::Unknown("Unknown response".to_owned())),
            Err(x) => Err(x),
        }
    } else {
        println!("0x{}", String::from(request_id));
        Ok(())
    }
}
