use crate::lib::canister_info::assets::AssetsCanisterInfo;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::waiter::waiter_with_timeout;
use candid::{CandidType, Decode, Encode};

use futures::future::try_join_all;
use ic_agent::Agent;
use ic_types::Principal;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;
use walkdir::{DirEntry, WalkDir};

const GET: &str = "get";
const CREATE_BATCH: &str = "create_batch";
const CREATE_CHUNK: &str = "create_chunk";
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

#[derive(Clone, Debug)]
struct AssetLocation {
    source: PathBuf,
    relative: PathBuf,
}

struct ChunkedAsset {
    asset_location: AssetLocation,
    chunk_ids: Vec<u128>,
}

async fn make_chunked_asset(
    agent: &Agent,
    canister_id: &Principal,
    timeout: Duration,
    batch_id: u128,
    asset_location: AssetLocation,
) -> DfxResult<ChunkedAsset> {
    let content = &std::fs::read(&asset_location.source)?;
    println!(
        "create chunks for {}",
        asset_location.source.to_string_lossy()
    );
    let chunks_futures: Vec<_> = content
        .chunks(MAX_CHUNK_SIZE)
        .map(|content| async move {
            let args = CreateChunkRequest { batch_id, content };
            let args = candid::Encode!(&args).expect("unable to encode create_chunk argument");
            println!("create chunk");
            agent
                .update(&canister_id, CREATE_CHUNK)
                .with_arg(args)
                .expire_after(timeout)
                .call_and_wait(waiter_with_timeout(timeout))
                .await
                .map_err(DfxError::from)
                .and_then(|response| {
                    candid::Decode!(&response, CreateChunkResponse)
                        .map_err(DfxError::from)
                        .map(|x| x.chunk_id)
                })
        })
        .collect();
    println!("await chunk creation");

    try_join_all(chunks_futures)
        .await
        .map(|chunk_ids| ChunkedAsset {
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
    let futs: Vec<_> = locs
        .into_iter()
        .map(|loc| async { make_chunked_asset(agent, canister_id, timeout, batch_id, loc).await })
        .collect();
    try_join_all(futs).await
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
    println!("created all chunks");

    // let mut futs: Vec<_> = vec![];
    // for loc in asset_locations {
    //     let y = make_chunked_asset(agent, &canister_id, timeout, batch_id, loc);
    //     futs.append(y);
    // }
    // let chunked_asset_futures: Vec<_> = asset_locations
    //     .into_iter()
    //     .map(|loc| async { make_chunked_asset(agent, &canister_id, timeout, batch_id, loc) })
    //     .collect();
    // let x: Vec<ChunkedAsset> = try_join_all(chunked_asset_futures).await.unwrap();

    let walker = WalkDir::new(output_assets_path).into_iter();
    for entry in walker {
        let entry = entry?;
        if entry.file_type().is_file() {
            let source = entry.path();
            let relative: &Path = source
                .strip_prefix(output_assets_path)
                .expect("cannot strip prefix");
            let content = &std::fs::read(&source)?;
            let path = relative.to_string_lossy().to_string();
            let blob = candid::Encode!(&path, &content)?;

            let method_name = String::from("store");

            agent
                .update(&canister_id, &method_name)
                .with_arg(&blob)
                .expire_after(timeout)
                .call_and_wait(waiter_with_timeout(timeout))
                .await?;
        }
    }
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
    let batch_id = create_batch_response.batch_id;
    Ok(batch_id)
}
