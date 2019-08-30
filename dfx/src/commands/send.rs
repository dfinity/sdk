use crate::lib::env::{Env};
use crate::lib::api_client::*;
use crate::lib::error::{DfxError, DfxResult};
use clap::{App, Arg, ArgMatches, SubCommand};
use futures::future::{err, ok, Future};
use tokio::runtime::Runtime;

const NAME_ARG: &str = "name";

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("send")
        .about(r#"Send a "Hello World" request to the canister 42."#)
        .arg(
            Arg::with_name(NAME_ARG)
                .help("The person to say hello to.")
                .required(true),
        )
}

pub fn exec(env: &'static Env, args: &ArgMatches<'_>) -> DfxResult {
    let name = args.value_of(NAME_ARG).unwrap();

    let query = query(
        &env.client,
        CanisterQueryCall {
            canister_id: 42,
            method_name: "dfn_msg greet".to_string(),
            arg: Some(Blob(Vec::from(name))),
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
