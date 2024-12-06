use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::clap::parsers;
use crate::util::print_idl_blob;
use anyhow::{anyhow, bail, Context};
use backoff::backoff::Backoff;
use backoff::ExponentialBackoff;
use candid::Principal;
use clap::Parser;
use ic_agent::agent::RequestStatusResponse;
use ic_agent::{AgentError, RequestId};
use pocket_ic::common::rest::{RawEffectivePrincipal, RawMessageId};
use pocket_ic::WasmResult;
use std::str::FromStr;

/// Requests the status of a call from a canister.
#[derive(Parser)]
pub struct RequestStatusOpts {
    /// Specifies the request identifier.
    /// The request identifier is an hexadecimal string starting with 0x.
    #[arg(value_parser = parsers::request_id_parser)]
    request_id: String,

    /// Specifies the name or id of the canister onto which the request was made.
    /// If the request was made to the Management canister, specify the id of the
    /// canister it is updating/querying.
    /// If the call was proxied by the wallet,
    /// i.e. a `dfx canister call --async --wallet=<ID>` flag,
    /// specify the wallet canister id.
    canister: String,

    /// Specifies the format for displaying the method's return result.
    #[arg(long, value_parser = ["idl", "raw", "pp"])]
    output: Option<String>,
}

pub async fn exec(env: &dyn Environment, opts: RequestStatusOpts) -> DfxResult {
    let request_id =
        RequestId::from_str(&opts.request_id[2..]).context("Invalid argument: request_id")?;
    let agent = env.get_agent();

    fetch_root_key_if_needed(env).await?;

    let callee_canister = opts.canister.as_str();
    let canister_id_store = env.get_canister_id_store()?;

    let canister_id = Principal::from_text(callee_canister)
        .or_else(|_| canister_id_store.get(callee_canister))?;

    let blob = if let Some(pocketic) = env.get_pocketic() {
        let msg_id = RawMessageId {
            effective_principal: RawEffectivePrincipal::CanisterId(canister_id.as_slice().to_vec()),
            message_id: request_id.as_slice().to_vec(),
        };
        let res = pocketic
            .await_call_no_ticks(msg_id)
            .await
            .map_err(|err| anyhow!("Canister call failed: {}", err))?;
        match res {
            WasmResult::Reply(data) => data,
            WasmResult::Reject(err) => bail!("Canister rejected: {}", err),
        }
    } else {
        let mut retry_policy = ExponentialBackoff::default();
        async {
            let mut request_accepted = false;
            loop {
                let (response, _cert) = agent
                    .request_status_raw(&request_id, canister_id)
                    .await
                    .context("Failed to fetch request status.")?;
                match response {
                    RequestStatusResponse::Replied(reply) => return Ok(reply.arg),
                    RequestStatusResponse::Rejected(response) => {
                        return Err(DfxError::new(AgentError::CertifiedReject(response)))
                    }
                    RequestStatusResponse::Unknown => (),
                    RequestStatusResponse::Received | RequestStatusResponse::Processing => {
                        // The system will return Unknown until the request is accepted
                        // and we generally cannot know how long that will take.
                        // State transitions between Received and Processing may be
                        // instantaneous. Therefore, once we know the request is accepted,
                        // we restart the waiter so the request does not time out.
                        if !request_accepted {
                            retry_policy.reset();
                            request_accepted = true;
                        }
                    }
                    RequestStatusResponse::Done => {
                        return Err(DfxError::new(AgentError::RequestStatusDoneNoReply(
                            String::from(request_id),
                        )))
                    }
                };

                let interval = retry_policy
                    .next_backoff()
                    .ok_or_else(|| DfxError::new(AgentError::TimeoutWaitingForResponse()))?;
                tokio::time::sleep(interval).await;
            }
        }
        .await
        .map_err(DfxError::from)?
    };

    let output_type = opts.output.as_deref();
    print_idl_blob(&blob, output_type, &None)?;
    Ok(())
}
