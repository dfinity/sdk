use crate::lib::api_client::{
    install_code, request_status, Client, QueryResponseReply, ReadResponse,
};
use crate::lib::canister_info::CanisterInfo;
use crate::lib::env::{ClientEnv, ProjectConfigEnv};
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::util::print_idl_blob;
use clap::{App, Arg, ArgMatches, SubCommand};
use ic_http_agent::{Blob, RequestId};
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("install")
        .about(UserMessage::InstallCanister.to_str())
        .arg(
            Arg::with_name("canister_name")
                .takes_value(true)
                .required_unless("all")
                .help(UserMessage::InstallCanisterName.to_str())
                .required(false),
        )
        .arg(
            Arg::with_name("all")
                .long("all")
                .required_unless("canister_name")
                .help(UserMessage::InstallAll.to_str())
                .takes_value(false),
        )
        .arg(
            Arg::with_name("async")
                .help(UserMessage::AsyncResult.to_str())
                .long("async")
                .takes_value(false),
        )
}

pub fn install_canister(client: &Client, canister_info: &CanisterInfo) -> DfxResult<RequestId> {
    let canister_id = canister_info.get_canister_id().ok_or_else(|| {
        DfxError::CannotFindBuildOutputForCanister(canister_info.get_name().to_owned())
    })?;

    eprintln!(
        "Installing code for canister {}, with canister_id {}",
        canister_info.get_name(),
        canister_id.to_text(),
    );

    let wasm_path = canister_info.get_output_wasm_path();
    let wasm = std::fs::read(wasm_path)?;

    let install = install_code(client.clone(), canister_id, Blob::from(wasm), None);

    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    let request_id = runtime.block_on(install)?;

    Ok(request_id)
}

const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
const RETRY_PAUSE: Duration = Duration::from_millis(100);

pub fn wait_on_request_status(client: &Client, request_id: RequestId) -> DfxResult {
    let mut runtime = Runtime::new().expect("Unable to create a runtime");

    let mut response;
    let start = Instant::now();
    // While the answer is `Unknown`, this means the client is still managing the message
    // (could be consensus, p2p or something else). We just wait a small time to let
    // the client work, and call again. We stop waiting after `REQUEST_TIMEOUT`.
    loop {
        response = runtime.block_on(request_status(client.clone(), request_id))?;
        match response {
            ReadResponse::Replied { reply } => {
                if let Some(QueryResponseReply { arg: blob }) = reply {
                    print_idl_blob(&blob)
                        .map_err(|e| DfxError::InvalidData(format!("Invalid IDL blob: {}", e)))?;
                }
                return Ok(())
            },
            ReadResponse::Rejected { reject_code, reject_message, } => {
                return Err(DfxError::ClientError(reject_code, reject_message))
            },
            ReadResponse::Pending => (),
            ReadResponse::Unknown => (),
        };
        if start.elapsed() > REQUEST_TIMEOUT {
            return Err(DfxError::TimeoutWaitingForResponse(
                request_id,
                REQUEST_TIMEOUT,
            ))
        };
        std::thread::sleep(RETRY_PAUSE);
    }
}

pub fn exec<T>(env: &T, args: &ArgMatches<'_>) -> DfxResult
where
    T: ClientEnv + ProjectConfigEnv,
{
    // Read the config.
    let config = env
        .get_config()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;

    let client = env.get_client();
    if let Some(canister_name) = args.value_of("canister_name") {
        let canister_info = CanisterInfo::load(config, canister_name)?;
        let request_id = install_canister(&client, &canister_info)?;

        if args.is_present("async") {
            eprint!("Request ID: ");
            println!("0x{}", String::from(request_id));
            Ok(())
        } else {
            wait_on_request_status(&client, request_id)
        }
    } else if args.is_present("all") {
        // Install all canisters.
        if let Some(canisters) = &config.get_config().canisters {
            for canister_name in canisters.keys() {
                let canister_info = CanisterInfo::load(config, canister_name)?;
                let request_id = install_canister(&client, &canister_info)?;

                if args.is_present("async") {
                    eprint!("Request ID: ");
                    println!("0x{}", String::from(request_id));
                } else {
                    wait_on_request_status(&client, request_id)?;
                }
            }
        }
        Ok(())
    } else {
        Err(DfxError::CanisterNameMissing())
    }
}
