use crate::commands::canister::install::wait_on_request_status;
use crate::lib::api_client::{call, query, QueryResponseReply, ReadResponse};
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::util::{load_idl_file, print_idl_blob};
use clap::{App, Arg, ArgMatches, SubCommand};
use ic_http_agent::{Blob, RequestId};
use serde_idl::{Encode, IDLArgs};
use std::io::Write;
use tokio::runtime::Runtime;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("call")
        .about(UserMessage::CallCanister.to_str())
        .arg(
            Arg::with_name("canister_name")
                .takes_value(true)
                .help(UserMessage::CanisterName.to_str())
                .required(true),
        )
        .arg(
            Arg::with_name("method_name")
                .help(UserMessage::MethodName.to_str())
                .required(true),
        )
        .arg(
            Arg::with_name("async")
                .help(UserMessage::AsyncResult.to_str())
                .long("async")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("query")
                .help(UserMessage::QueryCanister.to_str())
                .long("query")
                .conflicts_with("async")
                .conflicts_with("update")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("update")
                .help(UserMessage::UpdateCanisterArg.to_str())
                .long("update")
                .conflicts_with("async")
                .conflicts_with("query")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("type")
                .help(UserMessage::ArgumentType.to_str())
                .long("type")
                .takes_value(true)
                .requires("argument")
                .possible_values(&["string", "number", "idl", "raw"]),
        )
        .arg(
            Arg::with_name("argument")
                .help(UserMessage::ArgumentValue.to_str())
                .takes_value(true),
        )
}

pub fn read_response(
    response: ReadResponse<QueryResponseReply>,
    request_id: Option<RequestId>,
) -> DfxResult {
    match response {
        ReadResponse::Pending => {
            match request_id {
                None => eprintln!("Pending"),
                Some(request_id) => {
                    eprint!("Request ID: ");
                    std::io::stderr().flush()?;
                    println!("0x{}", String::from(request_id));
                }
            }
            Ok(())
        }
        ReadResponse::Replied { reply } => {
            if let Some(QueryResponseReply { arg: blob }) = reply {
                print_idl_blob(&blob)
                    .map_err(|e| DfxError::InvalidData(format!("Invalid IDL blob: {}", e)))?;
            }
            Ok(())
        }
        ReadResponse::Rejected {
            reject_code,
            reject_message,
        } => Err(DfxError::ClientError(reject_code, reject_message)),
        ReadResponse::Unknown => Err(DfxError::Unknown("Unknown response".to_owned())),
    }
}

pub fn exec(env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult {
    let config = env
        .get_config()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;

    let canister_name = args.value_of("canister_name").unwrap();
    let canister_info = CanisterInfo::load(&config, canister_name)?;
    // Read the config.
    let canister_id = canister_info.get_canister_id().ok_or_else(|| {
        DfxError::CannotFindBuildOutputForCanister(canister_info.get_name().to_owned())
    })?;
    let method_name = args
        .value_of("method_name")
        .ok_or_else(|| DfxError::InvalidArgument("method_name".to_string()))?;
    let arguments: Option<&str> = args.value_of("argument");
    let arg_type: Option<&str> = args.value_of("type");

    let idl_ast = load_idl_file(env, canister_info.get_output_idl_path());
    let is_query = if args.is_present("async") {
        false
    } else {
        let is_query_method =
            idl_ast.and_then(|ast| ast.get_method_type(&method_name).map(|f| f.is_query()));
        match is_query_method {
            Some(true) => !args.is_present("update"),
            Some(false) => {
                if args.is_present("query") {
                    return Err(DfxError::InvalidMethodCall(format!(
                        "{} is not a query method",
                        method_name
                    )));
                } else {
                    false
                }
            }
            None => args.is_present("query"),
        }
    };

    // Get the argument, get the type, convert the argument to the type and return
    // an error if any of it doesn't work.
    let arg_value = if let Some(a) = arguments {
        Some(Blob::from(match arg_type {
            Some("string") => Ok(Encode!(&a)),
            Some("number") => Ok(Encode!(&a.parse::<u64>().map_err(|e| {
                DfxError::InvalidArgument(format!(
                    "Argument is not a valid 64-bit unsigned integer: {}",
                    e
                ))
            })?)),
            Some("raw") => Ok(hex::decode(&a).map_err(|e| {
                DfxError::InvalidArgument(format!("Argument is not a valid hex string: {}", e))
            })?),
            Some("idl") | None => {
                let args: IDLArgs = a
                    .parse()
                    .map_err(|e| DfxError::InvalidArgument(format!("Invalid IDL: {}", e)))?;
                Ok(args.to_bytes().map_err(|e| {
                    DfxError::InvalidData(format!("Unable to convert IDL to bytes: {}", e))
                })?)
            }
            Some(v) => Err(DfxError::Unknown(format!("Invalid type: {}", v))),
        }?))
    } else {
        None
    };

    let client = env
        .get_client()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;
    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    if is_query {
        let future = query(
            client.clone(),
            canister_id,
            method_name.to_owned(),
            arg_value.map(Blob::from),
        );
        read_response(runtime.block_on(future)?, None)
    } else {
        let future = call(
            client.clone(),
            canister_id,
            method_name.to_owned(),
            arg_value,
        );
        let request_id: RequestId = runtime.block_on(future)?;
        if args.is_present("async") {
            eprint!("Request ID: ");
            println!("0x{}", String::from(request_id));
            Ok(())
        } else {
            wait_on_request_status(&client.clone(), request_id)
        }
    }
}
