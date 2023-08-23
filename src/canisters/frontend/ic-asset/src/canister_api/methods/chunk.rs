use crate::batch_upload::retryable::retryable;
use crate::batch_upload::semaphores::Semaphores;
use crate::canister_api::methods::method_names::CREATE_CHUNK;
use crate::canister_api::types::batch_upload::common::{CreateChunkRequest, CreateChunkResponse};
use crate::error::CreateChunkError;
use backoff::backoff::Backoff;
use backoff::ExponentialBackoffBuilder;
use candid::{Decode, Nat};
use ic_utils::Canister;
use std::time::Duration;

pub(crate) async fn create_chunk(
    canister: &Canister<'_>,
    batch_id: &Nat,
    content: &[u8],
    semaphores: &Semaphores,
) -> Result<Nat, CreateChunkError> {
    let _chunk_releaser = semaphores.create_chunk.acquire(1).await;
    let batch_id = batch_id.clone();
    let args = CreateChunkRequest { batch_id, content };
    let mut retry_policy = ExponentialBackoffBuilder::new()
        .with_initial_interval(Duration::from_secs(1))
        .with_max_interval(Duration::from_secs(16))
        .with_multiplier(2.0)
        .with_max_elapsed_time(Some(Duration::from_secs(300)))
        .build();

    loop {
        let builder = canister.update_(CREATE_CHUNK);
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
                canister.wait(request_id).await
            }
            Err(agent_err) => Err(agent_err),
        };

        match wait_result {
            Ok(response) => {
                // failure to decode the response is not retryable
                let response = Decode!(&response, CreateChunkResponse)
                    .map_err(CreateChunkError::DecodeCreateChunkResponse)?;
                return Ok(response.chunk_id);
            }
            Err(agent_err) if !retryable(&agent_err) => {
                return Err(CreateChunkError::CreateChunk(agent_err));
            }
            Err(agent_err) => match retry_policy.next_backoff() {
                Some(duration) => tokio::time::sleep(duration).await,
                None => return Err(CreateChunkError::CreateChunk(agent_err)),
            },
        }
    }
}
