use crate::lib::canister_info::assets::AssetsCanisterInfo;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::waiter::waiter_with_timeout;
use candid::{CandidType, Decode, Encode};

use delay::{Delay, Waiter};
use ic_agent::Agent;
use ic_types::Principal;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use walkdir::WalkDir;

const CREATE_BATCH: &str = "create_batch";
const CREATE_CHUNK: &str = "create_chunk";
const COMMIT_BATCH: &str = "commit_batch";
const MAX_CHUNK_SIZE: usize = 1_900_000;

#[derive(CandidType, Clone, Debug, Default, Serialize, Deserialize)]
struct CreateBatchRequest {}

#[derive(CandidType, Clone, Debug, Default, Serialize, Deserialize)]
struct CreateBatchResponse {
    batch_id: u128,
}

#[derive(CandidType, Clone, Debug, Default, Serialize, Deserialize)]
struct CreateChunkRequest<'a> {
    batch_id: u128,
    #[serde(with = "serde_bytes")]
    content: &'a [u8],
}

#[derive(CandidType, Clone, Debug, Default, Serialize, Deserialize)]
struct CreateChunkResponse {
    chunk_id: u128,
}

#[derive(CandidType, Clone, Debug, Default, Serialize, Deserialize)]
struct GetRequest {
    key: String,
    accept_encodings: Vec<String>,
}

#[derive(CandidType, Clone, Debug, Default, Serialize, Deserialize)]
struct GetResponse {
    contents: Vec<u8>,
    content_type: String,
    content_encoding: String,
}

#[derive(CandidType, Clone, Debug, Default, Serialize, Deserialize)]
struct CreateAssetArguments {
    key: String,
    content_type: String,
}
#[derive(CandidType, Clone, Debug, Default, Serialize, Deserialize)]
struct SetAssetContentArguments {
    key: String,
    content_encoding: String,
    chunk_ids: Vec<u128>,
}
#[derive(CandidType, Clone, Debug, Default, Serialize, Deserialize)]
struct UnsetAssetContentArguments {
    key: String,
    content_encoding: String,
}
#[derive(CandidType, Clone, Debug, Default, Serialize, Deserialize)]
struct DeleteAssetArguments {
    key: String,
}
#[derive(CandidType, Clone, Debug, Default, Serialize, Deserialize)]
struct ClearArguments {}

#[derive(CandidType, Clone, Debug, Serialize, Deserialize)]
enum BatchOperationKind {
    CreateAsset(CreateAssetArguments),

    SetAssetContent(SetAssetContentArguments),

    UnsetAssetContent(UnsetAssetContentArguments),

    DeleteAsset(DeleteAssetArguments),

    Clear(ClearArguments),
}

#[derive(CandidType, Clone, Debug, Default, Serialize, Deserialize)]
struct CommitBatchArguments {
    batch_id: u128,
    operations: Vec<BatchOperationKind>,
}

#[derive(Clone, Debug)]
struct AssetLocation {
    source: PathBuf,
    relative: PathBuf,
}

struct ChunkedAsset {
    asset_location: AssetLocation,
    chunk_ids: Vec<u128>,
}

async fn create_chunk(
    agent: &Agent,
    canister_id: &Principal,
    timeout: Duration,
    batch_id: u128,
    content: &[u8],
) -> DfxResult<u128> {
    let args = CreateChunkRequest { batch_id, content };
    let args = candid::Encode!(&args)?;

    let mut waiter = Delay::builder()
        .timeout(std::time::Duration::from_secs(30))
        .throttle(std::time::Duration::from_secs(1))
        .build();
    waiter.start();

    loop {
        match agent
            .update(&canister_id, CREATE_CHUNK)
            .with_arg(&args)
            .expire_after(timeout)
            .call_and_wait(waiter_with_timeout(timeout))
            .await
            .map_err(DfxError::from)
            .and_then(|response| {
                candid::Decode!(&response, CreateChunkResponse)
                    .map_err(DfxError::from)
                    .map(|x| x.chunk_id)
            }) {
            Ok(chunk_id) => {
                break Ok(chunk_id);
            }
            Err(agent_err) => match waiter.wait() {
                Ok(()) => {}
                Err(_) => break Err(agent_err),
            },
        }
    }
}

async fn make_chunked_asset(
    agent: &Agent,
    canister_id: &Principal,
    timeout: Duration,
    batch_id: u128,
    asset_location: AssetLocation,
) -> DfxResult<ChunkedAsset> {
    let content = &std::fs::read(&asset_location.source)?;

    // ?? doesn't work: rust lifetimes + task::spawn = tears
    // how to deal with lifetimes for agent and canister_id here
    // this function won't exit until after the task is joined...
    // let chunks_future_tasks: Vec<_> = content
    //     .chunks(MAX_CHUNK_SIZE)
    //     .map(|content| task::spawn(create_chunk(agent, canister_id, timeout, batch_id, content)))
    //     .collect();
    // println!("await chunk creation");
    // let but_lifetimes = try_join_all(chunks_future_tasks)
    //     .await?
    //     .into_iter()
    //     .collect::<DfxResult<Vec<u128>>>()
    //     .map(|chunk_ids| ChunkedAsset {
    //         asset_location,
    //         chunk_ids,
    //     });
    // ?? doesn't work

    // works (sometimes), does more work concurrently, but often doesn't work against bootstrap.
    // (connection stuck in odd idle state: all agent requests return "channel closed" error.)
    // let chunks_futures: Vec<_> = content
    //     .chunks(MAX_CHUNK_SIZE)
    //     .map(|content| create_chunk(agent, canister_id, timeout, batch_id, content))
    //     .collect();
    // println!("await chunk creation");
    //
    // try_join_all(chunks_futures)
    //     .await
    //     .map(|chunk_ids| ChunkedAsset {
    //         asset_location,
    //         chunk_ids,
    //     })
    // works (sometimes)

    let mut chunk_ids: Vec<u128> = vec![];
    let chunks = content.chunks(MAX_CHUNK_SIZE);
    let (num_chunks, _) = chunks.size_hint();
    let mut i: usize = 0;
    for data_chunk in chunks {
        i += 1;
        println!(
            "  {} {}/{} ({} bytes)",
            &asset_location.relative.to_string_lossy(),
            i,
            num_chunks,
            data_chunk.len()
        );
        chunk_ids.push(create_chunk(agent, canister_id, timeout, batch_id, data_chunk).await?);
    }
    Ok(ChunkedAsset {
        asset_location,
        chunk_ids,
    })
}

async fn make_chunked_assets(
    agent: &Agent,
    canister_id: &Principal,
    timeout: Duration,
    batch_id: u128,
    locs: Vec<AssetLocation>,
) -> DfxResult<Vec<ChunkedAsset>> {
    // this neat futures version works faster in parallel when it works,
    // but does not work often when connecting through the bootstrap.
    // let futs: Vec<_> = locs
    //     .into_iter()
    //     .map(|loc| make_chunked_asset(agent, canister_id, timeout, batch_id, loc))
    //     .collect();
    // try_join_all(futs).await
    let mut chunked_assets = vec![];
    for loc in locs {
        chunked_assets.push(make_chunked_asset(agent, canister_id, timeout, batch_id, loc).await?);
    }
    Ok(chunked_assets)
}

async fn commit_batch(
    agent: &Agent,
    canister_id: &Principal,
    timeout: Duration,
    batch_id: u128,
    chunked_assets: Vec<ChunkedAsset>,
) -> DfxResult {
    let operations: Vec<_> = chunked_assets
        .into_iter()
        .map(|chunked_asset| {
            let key = chunked_asset
                .asset_location
                .relative
                .to_string_lossy()
                .to_string();
            vec![
                BatchOperationKind::DeleteAsset(DeleteAssetArguments { key: key.clone() }),
                BatchOperationKind::CreateAsset(CreateAssetArguments {
                    key: key.clone(),
                    content_type: "application/octet-stream".to_string(),
                }),
                BatchOperationKind::SetAssetContent(SetAssetContentArguments {
                    key,
                    content_encoding: "identity".to_string(),
                    chunk_ids: chunked_asset.chunk_ids,
                }),
            ]
        })
        .flatten()
        .collect();
    let arg = CommitBatchArguments {
        batch_id,
        operations,
    };
    let arg = candid::Encode!(&arg)?;
    agent
        .update(&canister_id, COMMIT_BATCH)
        .with_arg(arg)
        .expire_after(timeout)
        .call_and_wait(waiter_with_timeout(timeout))
        .await?;
    Ok(())
}

pub async fn post_install_store_assets(
    info: &CanisterInfo,
    agent: &Agent,
    timeout: Duration,
) -> DfxResult {
    let assets_canister_info = info.as_info::<AssetsCanisterInfo>()?;
    let output_assets_path = assets_canister_info.get_output_assets_path();

    let asset_locations: Vec<AssetLocation> = WalkDir::new(output_assets_path)
        .into_iter()
        .filter_map(|r| {
            r.ok().filter(|entry| entry.file_type().is_file()).map(|e| {
                let source = e.path().to_path_buf();
                let relative = source
                    .strip_prefix(output_assets_path)
                    .expect("cannot strip prefix")
                    .to_path_buf();
                AssetLocation { source, relative }
            })
        })
        .collect();

    let canister_id = info.get_canister_id().expect("Could not find canister ID.");

    let batch_id = create_batch(agent, &canister_id, timeout).await?;

    let chunked_assets =
        make_chunked_assets(agent, &canister_id, timeout, batch_id, asset_locations).await?;

    commit_batch(agent, &canister_id, timeout, batch_id, chunked_assets).await?;

    Ok(())
}

async fn create_batch(
    agent: &Agent,
    canister_id: &Principal,
    timeout: Duration,
) -> DfxResult<u128> {
    let create_batch_args = CreateBatchRequest {};
    let response = agent
        .update(&canister_id, CREATE_BATCH)
        .with_arg(candid::Encode!(&create_batch_args)?)
        .expire_after(timeout)
        .call_and_wait(waiter_with_timeout(timeout))
        .await?;
    let create_batch_response = candid::Decode!(&response, CreateBatchResponse)?;
    Ok(create_batch_response.batch_id)
}
