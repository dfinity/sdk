use crate::lib::api_client::{call, request_status, QueryResponseReply, ReadResponse};
use crate::lib::env::ClientEnv;
use crate::lib::error::{DfxError, DfxResult};
use crate::util::clap::validators;
use clap::{App, Arg, ArgMatches, SubCommand};
use ic_http_agent::{Blob, CanisterId};
use tokio::runtime::Runtime;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("call")
        .about("Call a canister.")
        .arg(
            Arg::with_name("canister")
                .takes_value(true)
                .help("The canister ID (a number) to call.")
                .required(true)
                .validator(validators::is_canister_id),
        )
        .arg(
            Arg::with_name("method_name")
                .help("The method name file to use.")
                .required(true),
        )
        .arg(
            Arg::with_name("wait")
                .help("Wait for the result of the call, by polling the client.")
                .long("wait")
                .short("w")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("arguments")
                .help("Arguments to pass to the method.")
                .takes_value(true)
                .multiple(true),
        )
}

pub fn exec<T>(env: &T, args: &ArgMatches<'_>) -> DfxResult
where
    T: ClientEnv,
{
    // Read the config.
    let canister_id = args.value_of("canister").unwrap().parse::<CanisterId>()?;
    let method_name = args.value_of("method_name").unwrap();
    let arguments: Option<Vec<&str>> = args.values_of("arguments").map(|args| args.collect());

    let client = env.get_client();
    let install = call(
        client,
        canister_id,
        method_name.to_owned(),
        arguments.map(|args| Blob(Vec::from(args[0]))),
    );

    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    let request_id = runtime.block_on(install)?;

    if args.is_present("wait") {
        let request_status = request_status(env.get_client(), request_id);
        let mut runtime = Runtime::new().expect("Unable to create a runtime");
        match runtime.block_on(request_status) {
            Ok(ReadResponse::Pending) => {
                println!("Pending");
                Ok(())
            }
            Ok(ReadResponse::Replied { reply }) => {
                if let Some(QueryResponseReply { arg: Blob(blob) }) = reply {
                    println!("{}", String::from_utf8_lossy(&blob));
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
    } else {
        println!("0x{}", String::from(request_id));
        Ok(())
    }
}
