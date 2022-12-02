use std::time::Duration;

use crate::asset_canister::method_names::{COMMIT_BATCH, CREATE_BATCH};
use crate::asset_canister::protocol::{
    BatchOperationKind, CommitBatchArguments, CreateBatchRequest, CreateBatchResponse,
};
use crate::params::CanisterCallParams;
use crate::retryable::retryable;
use backoff::backoff::Backoff;
use backoff::ExponentialBackoffBuilder;
use candid::Nat;

pub(crate) async fn create_batch(
    canister_call_params: &CanisterCallParams<'_>,
) -> anyhow::Result<Nat> {
    let mut retry_policy = ExponentialBackoffBuilder::new()
        .with_initial_interval(Duration::from_secs(1))
        .with_max_interval(Duration::from_secs(16))
        .with_multiplier(2.0)
        .with_max_elapsed_time(Some(Duration::from_secs(300)))
        .build();

    let result = loop {
        let create_batch_args = CreateBatchRequest {};
        let response = canister_call_params
            .canister
            .update_(CREATE_BATCH)
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

pub(crate) async fn commit_batch(
    canister_call_params: &CanisterCallParams<'_>,
    batch_id: &Nat,
    operations: Vec<BatchOperationKind>,
) -> anyhow::Result<()> {
    let mut retry_policy = ExponentialBackoffBuilder::new()
        .with_initial_interval(Duration::from_secs(1))
        .with_max_interval(Duration::from_secs(16))
        .with_multiplier(2.0)
        .with_max_elapsed_time(Some(Duration::from_secs(300)))
        .build();

    let arg = CommitBatchArguments {
        batch_id,
        operations,
    };
    let result = loop {
        match canister_call_params
            .canister
            .update_(COMMIT_BATCH)
            .with_arg(&arg)
            .build()
            .call_and_wait()
            .await
        {
            Ok(()) => break Ok(()),
            Err(agent_err) if !retryable(&agent_err) => {
                break Err(agent_err);
            }
            Err(agent_err) => match retry_policy.next_backoff() {
                Some(duration) => tokio::time::sleep(duration).await,
                None => break Err(agent_err),
            },
        }
    }?;
    Ok(result)
}
