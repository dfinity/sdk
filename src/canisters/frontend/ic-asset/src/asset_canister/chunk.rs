use crate::asset_canister::method_names::CREATE_CHUNK;
use crate::asset_canister::protocol::{CreateChunkRequest, CreateChunkResponse};
use crate::convenience::waiter_with_timeout;
use crate::params::CanisterCallParams;
use crate::retryable::retryable;
use crate::semaphores::Semaphores;
use candid::{Decode, Nat};
use garcon::{Delay, Waiter};

pub(crate) async fn create_chunk(
    canister_call_params: &CanisterCallParams<'_>,
    batch_id: &Nat,
    content: &[u8],
    semaphores: &Semaphores,
) -> anyhow::Result<Nat> {
    let _chunk_releaser = semaphores.create_chunk.acquire(1).await;
    let batch_id = batch_id.clone();
    let args = CreateChunkRequest { batch_id, content };

    let mut waiter = Delay::builder()
        .with(Delay::count_timeout(30))
        .exponential_backoff_capped(
            std::time::Duration::from_secs(1),
            2.0,
            std::time::Duration::from_secs(16),
        )
        .build();
    waiter.start();

    loop {
        let builder = canister_call_params.canister.update_(CREATE_CHUNK);
        let builder = builder.with_arg(&args);
        let request_id_result = {
            let _releaser = semaphores.create_chunk_call.acquire(1).await;
            builder
                .build()
                .map(|result: (CreateChunkResponse,)| (result.0.chunk_id,))
                .call()
                .await
        };
        let wait_result = match request_id_result {
            Ok(request_id) => {
                let _releaser = semaphores.create_chunk_wait.acquire(1).await;
                canister_call_params
                    .canister
                    .wait(
                        request_id,
                        waiter_with_timeout(canister_call_params.timeout),
                        false,
                    )
                    .await
            }
            Err(err) => Err(err),
        };
        match wait_result {
            Ok(response) => {
                // failure to decode the response is not retryable
                break Decode!(&response, CreateChunkResponse)
                    .map_err(|e| anyhow::anyhow!("{}", e))
                    .map(|x| x.chunk_id);
            }
            Err(agent_err) if !retryable(&agent_err) => {
                break Err(anyhow::anyhow!("{}", agent_err));
            }
            Err(agent_err) => {
                if let Err(_waiter_err) = waiter.async_wait().await {
                    break Err(anyhow::anyhow!("{}", agent_err));
                }
            }
        }
    }
}
