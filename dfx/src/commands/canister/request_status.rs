use crate::lib::api_client::{request_status, QueryResponseReply, ReadResponse};
use crate::lib::env::ClientEnv;
use crate::lib::error::{DfxError, DfxResult};
use crate::util::clap::validators;
use crate::util::print_idl_blob;
use clap::{App, Arg, ArgMatches, SubCommand};
use ic_http_agent::RequestId;
use std::str::FromStr;
use tokio::runtime::Runtime;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("request-status")
        .about("Request the status of a call to a canister.")
        .arg(
            Arg::with_name("request_id")
                .takes_value(true)
                .help("The request ID to call. This is an hexadecimal string starting with 0x.")
                .required(true)
                .validator(validators::is_request_id),
        )
}

pub fn exec<T>(env: &T, args: &ArgMatches<'_>) -> DfxResult
where
    T: ClientEnv,
{
    let request_id = RequestId::from_str(&args.value_of("request_id").unwrap()[2..])?;
    let request_status = request_status(env.get_client(), request_id);
    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    match runtime.block_on(request_status) {
        Ok(ReadResponse::Pending) => {
            println!("Pending");
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
