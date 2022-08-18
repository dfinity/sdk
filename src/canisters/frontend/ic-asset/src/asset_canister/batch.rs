use crate::asset_canister::method_names::{COMMIT_BATCH, CREATE_BATCH};
use crate::asset_canister::protocol::{
    BatchOperationKind, CommitBatchArguments, CreateBatchRequest, CreateBatchResponse,
};
use crate::convenience::waiter_with_timeout;
use crate::params::CanisterCallParams;
use crate::retryable::retryable;
use candid::Nat;
use garcon::{Delay, Waiter};

pub(crate) async fn create_batch(
    canister_call_params: &CanisterCallParams<'_>,
) -> anyhow::Result<Nat> {
    let mut waiter = Delay::builder()
        .with(Delay::count_timeout(30))
        .exponential_backoff_capped(
            std::time::Duration::from_secs(1),
            2.0,
            std::time::Duration::from_secs(16),
        )
        .build();
    waiter.start();

    let result = loop {
        let create_batch_args = CreateBatchRequest {};
        let response = canister_call_params
            .canister
            .update_(CREATE_BATCH)
            .with_arg(&create_batch_args)
            .build()
            .map(|result: (CreateBatchResponse,)| (result.0.batch_id,))
            .call_and_wait(waiter_with_timeout(canister_call_params.timeout))
            .await;
        match response {
            Ok((batch_id,)) => break Ok(batch_id),
            Err(agent_err) if !retryable(&agent_err) => {
                break Err(agent_err);
            }
            Err(agent_err) => {
                if let Err(_waiter_err) = waiter.async_wait().await {
                    break Err(agent_err);
                }
            }
        };
    }?;
    Ok(result)
}

pub(crate) async fn commit_batch(
    canister_call_params: &CanisterCallParams<'_>,
    batch_id: &Nat,
    operations: Vec<BatchOperationKind>,
) -> anyhow::Result<()> {
    let mut waiter = Delay::builder()
        .with(Delay::count_timeout(30))
        .exponential_backoff_capped(
            std::time::Duration::from_secs(1),
            2.0,
            std::time::Duration::from_secs(16),
        )
        .build();
    waiter.start();

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
            .call_and_wait(waiter_with_timeout(canister_call_params.timeout))
            .await
        {
            Ok(()) => break Ok(()),
            Err(agent_err) if !retryable(&agent_err) => {
                break Err(agent_err);
            }
            Err(agent_err) => {
                if let Err(_waiter_err) = waiter.async_wait().await {
                    break Err(agent_err);
                }
            }
        }
    }?;
    Ok(result)
}
