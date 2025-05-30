use crate::batch_upload::retryable::retryable;
use crate::batch_upload::semaphores::Semaphores;
use crate::canister_api::methods::method_names::CREATE_CHUNK;
use crate::canister_api::types::batch_upload::common::{
    CreateChunkRequest, CreateChunkResponse, CreateChunksRequest, CreateChunksResponse,
};
use crate::error::CreateChunkError;
use crate::AssetSyncProgressRenderer;
use backoff::backoff::Backoff;
use backoff::ExponentialBackoffBuilder;
use candid::{Decode, Nat};
use ic_agent::agent::CallResponse;
use ic_utils::Canister;
use std::time::Duration;

use super::method_names::CREATE_CHUNKS;

pub(crate) async fn create_chunk(
    canister: &Canister<'_>,
    batch_id: &Nat,
    content: &[u8],
    semaphores: &Semaphores,
    progress: Option<&dyn AssetSyncProgressRenderer>,
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
        let builder = canister.update(CREATE_CHUNK);
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
            Ok(resp) => match resp {
                CallResponse::Response(r) => Ok(r),
                CallResponse::Poll(id) => {
                    let _releaser = semaphores.create_chunk_wait.acquire(1).await;
                    canister
                        .wait(&id)
                        .await
                        .and_then(|bytes| Ok((Decode!(&bytes, CreateChunkResponse)?.chunk_id,)))
                }
            },
            Err(agent_err) => Err(agent_err),
        };

        match wait_result {
            Ok((chunk_id,)) => {
                if let Some(progress) = progress {
                    progress.add_uploaded_bytes(content.len());
                }
                return Ok(chunk_id);
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

pub(crate) async fn create_chunks(
    canister: &Canister<'_>,
    batch_id: &Nat,
    content: Vec<Vec<u8>>,
    semaphores: &Semaphores,
    progress: Option<&dyn AssetSyncProgressRenderer>,
) -> Result<Vec<Nat>, CreateChunkError> {
    let content_byte_len = content.iter().fold(0, |acc, x| acc + x.len());
    let _chunk_releaser = semaphores.create_chunk.acquire(1).await;
    let batch_id = batch_id.clone();
    let args = CreateChunksRequest { batch_id, content };
    let mut retry_policy = ExponentialBackoffBuilder::new()
        .with_initial_interval(Duration::from_secs(1))
        .with_max_interval(Duration::from_secs(16))
        .with_multiplier(2.0)
        .with_max_elapsed_time(Some(Duration::from_secs(300)))
        .build();

    loop {
        let builder = canister.update(CREATE_CHUNKS);
        let builder = builder.with_arg(&args);
        let request_id_result = {
            let _releaser = semaphores.create_chunk_call.acquire(1).await;
            builder
                .build()
                .map(|result: (CreateChunksResponse,)| (result.0.chunk_ids,))
                .call()
                .await
        };

        let wait_result = match request_id_result {
            Ok(resp) => match resp {
                CallResponse::Response(r) => Ok(r),
                CallResponse::Poll(id) => {
                    let _releaser = semaphores.create_chunk_wait.acquire(1).await;
                    canister
                        .wait(&id)
                        .await
                        .and_then(|bytes| Ok((Decode!(&bytes, CreateChunksResponse)?.chunk_ids,)))
                }
            },
            Err(agent_err) => Err(agent_err),
        };

        match wait_result {
            Ok((chunk_ids,)) => {
                if let Some(progress) = progress {
                    progress.add_uploaded_bytes(content_byte_len);
                }
                return Ok(chunk_ids);
            }
            Err(agent_err) if !retryable(&agent_err) => {
                return Err(CreateChunkError::CreateChunks(agent_err));
            }
            Err(agent_err) => match retry_policy.next_backoff() {
                Some(duration) => tokio::time::sleep(duration).await,
                None => return Err(CreateChunkError::CreateChunks(agent_err)),
            },
        }
    }
}
