use crate::lib::api_client::{query, QueryResponseReply, ReadResponse};
use crate::lib::env::ClientEnv;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::util::clap::validators;
use crate::util::print_idl_blob;
use clap::{App, Arg, ArgMatches, SubCommand};
use ic_http_agent::{Blob, CanisterId};
use serde_idl::Encode;
use tokio::runtime::Runtime;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("query")
        .about(UserMessage::QueryCanister.to_str())
        .arg(
            Arg::with_name("deployment_id")
                .takes_value(true)
                .help(UserMessage::CanisterId.to_str())
                .required(true)
                .validator(validators::is_canister_id),
        )
        .arg(
            Arg::with_name("method_name")
                .help(UserMessage::MethodName.to_str())
                .required(true),
        )
        .arg(
            Arg::with_name("argument")
                .help(UserMessage::ArgumentValue.to_str())
                .takes_value(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("type")
                .help(UserMessage::ArgumentType.to_str())
                .long("type")
                .takes_value(true)
                .requires("argument")
                .possible_values(&["string", "number"]),
        )
}

pub fn exec<T>(env: &T, args: &ArgMatches<'_>) -> DfxResult
where
    T: ClientEnv,
{
    // Read the config.
    let canister_id = args
        .value_of("deployment_id")
        .unwrap()
        .parse::<CanisterId>()?;
    let method_name = args.value_of("method_name").unwrap();
    let arguments: Option<&str> = args.value_of("argument");
    let arg_type: Option<&str> = args.value_of("type");

    let arg_value = if let Some(a) = arguments {
        Some(match arg_type {
            Some("string") => Ok(Encode!(&a)),
            Some("number") => Ok(Encode!(&a.parse::<u64>()?)),
            Some(v) => Err(DfxError::Unknown(format!("Invalid type: {}", v))),
            None => Err(DfxError::Unknown("Must specify a type.".to_owned())),
        }?)
    } else {
        None
    };

    let client = env.get_client();
    let install = query(
        client,
        canister_id,
        method_name.to_owned(),
        arg_value.map(Blob::from),
    );

    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    match runtime.block_on(install) {
        Ok(ReadResponse::Pending) => {
            eprintln!("Pending");
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
        // TODO(SDK-446): remove this when moving api_client to ic_http_agent.
        Ok(ReadResponse::Unknown) => Err(DfxError::Unknown("Unknown response".to_owned())),
        Err(x) => Err(x),
    }
}
