use std::time::Duration;

use crate::asset_canister::method_names::{COMMIT_BATCH, CREATE_BATCH};
use crate::asset_canister::protocol::{
    CommitBatchArguments, CreateBatchRequest, CreateBatchResponse,
};
use crate::retryable::retryable;
use anyhow::bail;
use backoff::backoff::Backoff;
use backoff::ExponentialBackoffBuilder;
use candid::Nat;
use ic_utils::Canister;

pub(crate) async fn create_batch(canister: &Canister<'_>) -> anyhow::Result<Nat> {
    let mut retry_policy = ExponentialBackoffBuilder::new()
        .with_initial_interval(Duration::from_secs(1))
        .with_max_interval(Duration::from_secs(16))
        .with_multiplier(2.0)
        .with_max_elapsed_time(Some(Duration::from_secs(300)))
        .build();

    let result = loop {
        let create_batch_args = CreateBatchRequest {};
        let response = canister
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
    canister: &Canister<'_>,
    arg: CommitBatchArguments,
) -> anyhow::Result<()> {
    let mut retry_policy = ExponentialBackoffBuilder::new()
        .with_initial_interval(Duration::from_secs(1))
        .with_max_interval(Duration::from_secs(16))
        .with_multiplier(2.0)
        .with_max_elapsed_time(Some(Duration::from_secs(300)))
        .build();

    loop {
        match canister
            .update_(COMMIT_BATCH)
            .with_arg(&arg)
            .build()
            .call_and_wait()
            .await
        {
            Ok(()) => return Ok(()),
            Err(agent_err) if !retryable(&agent_err) => {
                bail!(agent_err);
            }
            Err(agent_err) => match retry_policy.next_backoff() {
                Some(duration) => tokio::time::sleep(duration).await,
                None => bail!(agent_err),
            },
        }
    }
}
