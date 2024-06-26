use crate::batch_upload::retryable::retryable;
use crate::canister_api::methods::method_names::{
    COMMIT_BATCH, COMPUTE_EVIDENCE, CREATE_BATCH, PROPOSE_COMMIT_BATCH,
};
use crate::canister_api::types::batch_upload::common::{
    ComputeEvidenceArguments, CreateBatchRequest, CreateBatchResponse,
};
use backoff::backoff::Backoff;
use backoff::ExponentialBackoffBuilder;
use candid::{CandidType, Nat};
use ic_agent::AgentError;
use ic_utils::Canister;
use serde_bytes::ByteBuf;
use std::time::Duration;

pub(crate) async fn create_batch(canister: &Canister<'_>) -> Result<Nat, AgentError> {
    let mut retry_policy = ExponentialBackoffBuilder::new()
        .with_initial_interval(Duration::from_secs(1))
        .with_max_interval(Duration::from_secs(16))
        .with_multiplier(2.0)
        .with_max_elapsed_time(Some(Duration::from_secs(300)))
        .build();

    let result = loop {
        let create_batch_args = CreateBatchRequest {};
        let response = canister
            .update(CREATE_BATCH)
            .with_arg(&create_batch_args)
            .build()
            .map(|result: (CreateBatchResponse,)| (result.0.batch_id,))
            .call_and_wait()
            .await;
        match response {
            Ok((batch_id,)) => break Ok(batch_id),
            Err(agent_err) if !retryable(&agent_err) => {
                break Err(agent_err);
            }
            Err(agent_err) => match retry_policy.next_backoff() {
                Some(duration) => tokio::time::sleep(duration).await,
                None => break Err(agent_err),
            },
        };
    }?;
    Ok(result)
}

pub(crate) async fn submit_commit_batch<T: CandidType + Sync>(
    canister: &Canister<'_>,
    method_name: &str,
    arg: T, // CommitBatchArguments_{v0,v1,etc}
) -> Result<(), AgentError> {
    let mut retry_policy = ExponentialBackoffBuilder::new()
        .with_initial_interval(Duration::from_secs(1))
        .with_max_interval(Duration::from_secs(16))
        .with_multiplier(2.0)
        .with_max_elapsed_time(Some(Duration::from_secs(300)))
        .build();

    loop {
        match canister
            .update(method_name)
            .with_arg(&arg)
            .build()
            .call_and_wait()
            .await
        {
            Ok(()) => return Ok(()),
            Err(agent_err) if !retryable(&agent_err) => {
                return Err(agent_err);
            }
            Err(agent_err) => match retry_policy.next_backoff() {
                Some(duration) => tokio::time::sleep(duration).await,
                None => return Err(agent_err),
            },
        }
    }
}

pub(crate) async fn commit_batch<T: CandidType + Sync>(
    canister: &Canister<'_>,
    arg: T, // CommitBatchArguments_{v0,v1,etc}
) -> Result<(), AgentError> {
    submit_commit_batch(canister, COMMIT_BATCH, arg).await
}

pub(crate) async fn propose_commit_batch<T: CandidType + Sync>(
    canister: &Canister<'_>,
    arg: T, // CommitBatchArguments_{v0,v1,etc}
) -> Result<(), AgentError> {
    submit_commit_batch(canister, PROPOSE_COMMIT_BATCH, arg).await
}

pub async fn compute_evidence(
    canister: &Canister<'_>,
    arg: &ComputeEvidenceArguments,
) -> Result<Option<ByteBuf>, AgentError> {
    let mut retry_policy = ExponentialBackoffBuilder::new()
        .with_initial_interval(Duration::from_secs(1))
        .with_max_interval(Duration::from_secs(16))
        .with_multiplier(2.0)
        .with_max_elapsed_time(Some(Duration::from_secs(300)))
        .build();

    loop {
        match canister
            .update(COMPUTE_EVIDENCE)
            .with_arg(arg)
            .build()
            .map(|result: (Option<ByteBuf>,)| (result.0,))
            .call_and_wait()
            .await
        {
            Ok(x) => return Ok(x.0),
            Err(agent_err) if !retryable(&agent_err) => {
                return Err(agent_err);
            }
            Err(agent_err) => match retry_policy.next_backoff() {
                Some(duration) => tokio::time::sleep(duration).await,
                None => return Err(agent_err),
            },
        }
    }
}
