use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::lib::waiter::create_waiter;
use crate::util::clap::validators;
use crate::util::print_idl_blob;
use clap::{App, Arg, ArgMatches, SubCommand};
use delay::Waiter;
use ic_agent::{AgentError, Replied, RequestId, RequestStatusResponse};
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

    let mut waiter = create_waiter();

    let Replied::CallReplied(blob) = runtime
        .block_on(async {
            waiter.start();
            loop {
                match agent.request_status_raw(&request_id).await? {
                    RequestStatusResponse::Replied { reply } => return Ok(reply),
                    RequestStatusResponse::Rejected {
                        reject_code,
                        reject_message,
                    } => {
                        return Err(DfxError::AgentError(AgentError::ReplicaError {
                            reject_code,
                            reject_message,
                        }))
                    }
                    RequestStatusResponse::Unknown => (),
                    RequestStatusResponse::Received => (),
                    RequestStatusResponse::Processing => (),
                };

                waiter
                    .wait()
                    .map_err(|_| DfxError::AgentError(AgentError::TimeoutWaitingForResponse()))?;
            }
        })
        .map_err(DfxError::from)?;
    print_idl_blob(&blob, None, &None)
        .map_err(|e| DfxError::InvalidData(format!("Invalid IDL blob: {}", e)))?;
    Ok(())
}
