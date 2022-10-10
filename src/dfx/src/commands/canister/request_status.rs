use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::waiter::waiter_with_exponential_backoff;
use crate::util::clap::validators;
use crate::util::print_idl_blob;

use anyhow::{anyhow, Context};
use candid::Principal;
use clap::Parser;
use garcon::Waiter;
use ic_agent::agent::{Replied, RequestStatusResponse};
use ic_agent::{AgentError, RequestId};
use std::str::FromStr;

/// Requests the status of a call from a canister.
#[derive(Parser)]
pub struct RequestStatusOpts {
    /// Specifies the request identifier.
    /// The request identifier is an hexadecimal string starting with 0x.
    #[clap(validator(validators::is_request_id))]
    request_id: String,

    /// Specifies the name or id of the canister onto which the request was made.
    /// If the request was made to the Management canister, specify the id of the
    /// canister it is updating/querying.
    /// If the call was proxied by the wallet,
    /// i.e. a `dfx canister call --async --wallet=<ID>` flag,
    /// specify the wallet canister id.
    canister: String,

    /// Specifies the format for displaying the method's return result.
    #[clap(long,
        possible_values(&["idl", "raw", "pp"]))]
    output: Option<String>,
}

pub async fn exec(env: &dyn Environment, opts: RequestStatusOpts) -> DfxResult {
    let request_id =
        RequestId::from_str(&opts.request_id[2..]).context("Invalid argument: request_id")?;
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    fetch_root_key_if_needed(env).await?;

    let callee_canister = opts.canister.as_str();
    let canister_id_store = CanisterIdStore::for_env(env)?;

    let canister_id = Principal::from_text(callee_canister)
        .or_else(|_| canister_id_store.get(callee_canister))?;

    let mut waiter = waiter_with_exponential_backoff();
    let Replied::CallReplied(blob) = async {
        waiter.start();
        let mut request_accepted = false;
        loop {
            match agent
                .request_status_raw(&request_id, canister_id, false)
                .await
                .context("Failed to fetch request status.")?
            {
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

    let output_type = opts.output.as_deref();
    print_idl_blob(&blob, output_type, &None)?;
    Ok(())
}
