use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::waiter::waiter_with_exponential_backoff;
use crate::util::clap::validators;
use crate::util::print_idl_blob;

use anyhow::{anyhow, Context};
use clap::Clap;
use delay::Waiter;
use ic_agent::agent::{Replied, RequestStatusResponse};
use ic_agent::{AgentError, RequestId};
use std::str::FromStr;

/// Requests the status of a specified call from a canister.
#[derive(Clap)]
pub struct RequestStatusOpts {
    /// Specifies the request identifier.
    /// The request identifier is an hexadecimal string starting with 0x.
    #[clap(validator(validators::is_request_id))]
    request_id: String,
}

pub async fn exec(env: &dyn Environment, opts: RequestStatusOpts) -> DfxResult {
    let request_id =
        RequestId::from_str(&opts.request_id[2..]).context("Invalid argument: request_id")?;
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    fetch_root_key_if_needed(env).await?;

    let mut waiter = waiter_with_exponential_backoff();
    let Replied::CallReplied(blob) = async {
        waiter.start();
        let mut request_accepted = false;
        loop {
            match agent.request_status_raw(&request_id).await? {
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
                RequestStatusResponse::Received | RequestStatusResponse::Processing => {
                    // The system will return Unknown until the request is accepted
                    // and we generally cannot know how long that will take.
                    // State transitions between Received and Processing may be
                    // instantaneous. Therefore, once we know the request is accepted,
                    // we restart the waiter so the request does not time out.
                    if !request_accepted {
                        waiter
                            .restart()
                            .map_err(|_| DfxError::new(AgentError::WaiterRestartError()))?;
                        request_accepted = true;
                    }
                }
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
    }
    .await
    .map_err(DfxError::from)?;
    print_idl_blob(&blob, None, &None).context("Invalid data: Invalid IDL blob.")?;
    Ok(())
}
