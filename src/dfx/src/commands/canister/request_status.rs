use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::waiter::waiter_with_timeout;
use crate::util::clap::validators;
use crate::util::{expiry_duration, print_idl_blob};

use anyhow::{anyhow, Context};
use clap::{App, ArgMatches, Clap, FromArgMatches, IntoApp};
use delay::Waiter;
use ic_agent::agent::{Replied, RequestStatusResponse};
use ic_agent::{AgentError, RequestId};
use std::str::FromStr;
use tokio::runtime::Runtime;

/// Requests the status of a specified call from a canister.
#[derive(Clap)]
#[clap(name("request-status"))]
pub struct RequestStatusOpts {
    /// Specifies the request identifier.
    /// The request identifier is an hexadecimal string starting with 0x.
    #[clap(validator(validators::is_request_id))]
    request_id: String,
}

pub fn construct() -> App<'static> {
    RequestStatusOpts::into_app()
}

pub fn exec(env: &dyn Environment, args: &ArgMatches) -> DfxResult {
    let opts = RequestStatusOpts::from_arg_matches(args);
    let request_id =
        RequestId::from_str(&opts.request_id[2..]).context("Invalid data: request_id")?;
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;
    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    let timeout = expiry_duration();

    let mut waiter = waiter_with_timeout(timeout);
    let Replied::CallReplied(blob) = runtime
        .block_on(async {
            waiter.start();
            loop {
                match agent.request_status_raw(&request_id, None).await? {
                    RequestStatusResponse::Replied { reply } => return Ok(reply),
                    RequestStatusResponse::Rejected {
                        reject_code,
                        reject_message,
                    } => {
                        return Err(DfxError::new(AgentError::ReplicaError {
                            reject_code,
                            reject_message,
                        }))
                    }
                    RequestStatusResponse::Unknown => (),
                    RequestStatusResponse::Received => (),
                    RequestStatusResponse::Processing => (),
                    RequestStatusResponse::Done => {
                        return Err(DfxError::new(AgentError::RequestStatusDoneNoReply(
                            String::from(request_id),
                        )))
                    }
                };

                waiter
                    .wait()
                    .map_err(|_| DfxError::new(AgentError::TimeoutWaitingForResponse()))?;
            }
        })
        .map_err(DfxError::from)?;
    print_idl_blob(&blob, None, &None).context("Invalid data: Invalid IDL blob.")?;
    Ok(())
}
