use crate::lib::api_client::request_status;
use crate::lib::env::ClientEnv;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::util::clap::validators;
use clap::{App, Arg, ArgMatches, SubCommand};
use ic_http_agent::RequestId;
use std::str::FromStr;
use tokio::runtime::Runtime;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("request-status")
        .about(UserMessage::RequestCallStatus.to_str())
        .arg(
            Arg::with_name("request_id")
                .takes_value(true)
                .help(UserMessage::RequestId.to_str())
                .required(true)
                .validator(validators::is_request_id),
        )
}

pub fn exec<T>(env: &T, args: &ArgMatches<'_>) -> DfxResult
where
    T: ClientEnv,
{
    let request_id = RequestId::from_str(&args.value_of("request_id").unwrap()[2..])
        .map_err(|e| DfxError::InvalidArgument(format!("Invalid request ID: {:?}", e)))?; // FIXME Default formatter for RequestIdFromStringError
    let request_status = request_status(env.get_client(), request_id);
    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    let response = runtime.block_on(request_status)?;
    crate::commands::canister::call::read_response(response, Some(request_id))
}
