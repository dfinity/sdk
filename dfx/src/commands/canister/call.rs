use crate::config::dfinity::Config;
use crate::lib::api_client::*;
use crate::lib::error::{DfxError, DfxResult};
use clap::{App, Arg, ArgMatches, SubCommand};
use futures::future::{err, ok, Future};
use tokio::runtime::Runtime;

pub fn available() -> bool {
    true
}

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("call")
        .about(r#"Send a "Hello World" request to the canister."#)
        .arg(
            Arg::with_name("canister")
                .help("The ID of the authentication to add.")
                .required(true),
        )
        .arg(
            Arg::with_name("method_name")
                .help("The method to call.")
                .required(true),
        )
        .arg(Arg::with_name("args").help("The argument."))
}

pub fn exec(args: &ArgMatches<'_>) -> DfxResult {
    let name = args.value_of("args").unwrap();
    let method_name = args.value_of("method_name").unwrap();

    let url = match args.value_of("network") {
        Some(url) => url.to_string(),
        None => {
            let config = Config::from_current_dir()?;
            let default_address = &config.get_config().get_defaults().get_start().address;
            let default_address = default_address
                .clone()
                .unwrap_or_else(|| "127.0.0.1".to_owned());
            let address = args
                .value_of("address")
                .unwrap_or_else(|| default_address.as_str());

            let port = match args.value_of("port") {
                Some(port) => port.parse::<u16>()?,
                None => config
                    .get_config()
                    .get_defaults()
                    .get_start()
                    .get_port(8080),
            };

            format!("http://{}:{}", address, port)
        }
    };

    let client = Client::new(ClientConfig {
        url: url.to_string(),
    });
    let canister_id = args.value_of("canister").unwrap().parse::<u64>()?;

    let query = query(
        client,
        CanisterQueryCall {
            canister_id,
            method_name: method_name.to_string(),
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
