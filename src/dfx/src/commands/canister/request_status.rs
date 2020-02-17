use crate::commands::canister::create_waiter;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::util::clap::validators;
use crate::util::print_idl_blob;
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

pub fn exec(env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult {
    let request_id = RequestId::from_str(
        &args
            .value_of("request_id")
            .ok_or_else(|| DfxError::InvalidArgument("request_id".to_string()))?[2..],
    )
    .map_err(|_| DfxError::InvalidArgument("request_id".to_owned()))?;

    let agent = env
        .get_agent()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;
    let mut runtime = Runtime::new().expect("Unable to create a runtime");

    if let Some(blob) = runtime
        .block_on(agent.request_status_and_wait(&request_id, create_waiter()))
        .map_err(DfxError::from)?
    {
        print_idl_blob(&blob)
            .map_err(|e| DfxError::InvalidData(format!("Invalid IDL blob: {}", e)))?;
    }
    Ok(())
}
