use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::clap::parsers;
use crate::util::print_idl_blob;
use anyhow::Context;
use backoff::backoff::Backoff;
use backoff::ExponentialBackoff;
use candid::Principal;
use clap::Parser;
use ic_agent::agent::RequestStatusResponse;
use ic_agent::agent::{RejectCode, RejectResponse};
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

    let mut retry_policy = ExponentialBackoff::default();
    let blob = async {
        let mut request_accepted = false;
        loop {
            if let Some(pocketic_handle) = env.get_pocketic() {
                let msg_id = RawMessageId {
                    effective_principal: RawEffectivePrincipal::CanisterId(
                        canister_id.as_slice().to_vec(),
                    ),
                    message_id: request_id.as_slice().to_vec(),
                };
                if let Some(status) = pocketic_handle.ingress_status(msg_id).await {
                    match status {
                        Ok(WasmResult::Reply(data)) => return Ok(data),
                        Ok(WasmResult::Reject(reject_message)) => {
                            // https://github.com/dfinity/ic/blob/2845f3a1e81eb9eddc0e5541c5565bf6cff898d0/rs/canonical_state/src/lazy_tree_conversion.rs#L509-L513
                            let reject = RejectResponse {
                                reject_code: RejectCode::CanisterReject,
                                reject_message,
                                error_code: Some("IC0406".to_string()), // https://github.com/dfinity/ic/blob/2845f3a1e81eb9eddc0e5541c5565bf6cff898d0/rs/types/error_types/src/lib.rs#L204
                            };
                            return Err(DfxError::new(AgentError::CertifiedReject(reject)));
                        }
                        Err(user_error) => {
                            let error_code_as_u64 = user_error.code as u64;
                            let derived_reject_code = error_code_as_u64 / 100;
                            let reject = RejectResponse {
                                reject_code: derived_reject_code.try_into().unwrap(),
                                reject_message: user_error.description,
                                error_code: Some(user_error.code.to_string()),
                            };
                            return Err(DfxError::new(AgentError::CertifiedReject(reject)));
                        }
                    }
                }
            } else {
                match agent
                    .request_status_raw(&request_id, canister_id)
                    .await
                    .context("Failed to fetch request status.")?
                {
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
            }

            let interval = retry_policy
                .next_backoff()
                .ok_or_else(|| DfxError::new(AgentError::TimeoutWaitingForResponse()))?;
            tokio::time::sleep(interval).await;
        }
    }
    .await
    .map_err(DfxError::from)?;

    let output_type = opts.output.as_deref();
    print_idl_blob(&blob, output_type, &None)?;
    Ok(())
}
