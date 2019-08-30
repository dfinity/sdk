use crate::lib::api_client::*;
use crate::lib::error::{DfxError, DfxResult};
use clap::{App, Arg, ArgMatches, SubCommand};
use futures::future::{err, ok, Future};
use tokio::runtime::Runtime;

const HOST_ARG: &str = "host";
const NAME_ARG: &str = "name";

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("send")
        .about(r#"Send a "Hello World" request to the canister 42."#)
        .arg(
            Arg::with_name(HOST_ARG)
                .help("The host (with port) to send the query to.")
                .required(true),
        )
        .arg(
            Arg::with_name(NAME_ARG)
                .help("The person to say hello to.")
                .required(true),
        )
}

pub fn exec(args: &ArgMatches<'_>) -> DfxResult {
    let name = args.value_of(NAME_ARG).unwrap();
    let url = args.value_of(HOST_ARG).unwrap();

    let client = Client::new(ClientConfig {
        url: url.to_string(),
    });

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
