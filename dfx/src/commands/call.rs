use crate::lib::api_client::*;
use crate::lib::env::ClientEnv;
use crate::lib::error::{DfxError, DfxResult};
use clap::{App, Arg, ArgMatches, SubCommand};
use futures::future::{err, ok, Future};
use tokio::runtime::Runtime;

const HOST_ARG: &str = "host";
const NAME_ARG: &str = "name";

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("call")
        .about(r#"Call a "greet" function on the canister with ID 42."#)
        .arg(
            Arg::with_name(HOST_ARG)
                .help("The host (with port) to send the request to.")
                .required(true),
        )
        .arg(
            Arg::with_name(NAME_ARG)
                .help("The person to say hello to.")
                .required(true),
        )
}

pub fn exec<T>(env: &T, args: &ArgMatches<'_>) -> DfxResult
where
    T: ClientEnv,
{
    let name = args.value_of(NAME_ARG).unwrap();
    let client = env.get_client();

    let query = query(
        client,
        CanisterQueryCall {
            canister_id: 42,
            method_name: "greet".to_string(),
            arg: Blob(Vec::from(name)),
        },
    )
    .and_then(|r| match r {
        Response::Accepted => {
            println!("Accepted");
            ok(())
        }
        Response::Replied {
            reply: QueryResponseReply { arg: Blob(blob) },
        } => {
            println!("{}", String::from_utf8_lossy(&blob));
            ok(())
        }
        Response::Rejected {
            reject_code,
            reject_message,
        } => err(DfxError::ClientError(reject_code, reject_message)),
        Response::Unknown => err(DfxError::Unknown("Unknown response".to_owned())),
    });

    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(query)
}
