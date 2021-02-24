use crate::lib::canister_info::assets::AssetsCanisterInfo;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::error::DfxResult;
use crate::lib::waiter::waiter_with_timeout;
use candid::{CandidType, Decode, Encode};

use ic_agent::Agent;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;
use walkdir::{DirEntry, WalkDir};

const GET: &str = "get";
const CREATE_BATCH: &str = "create_batch";

#[derive(CandidType, Clone, Debug, Default, Serialize, Deserialize)]
struct CreateBatchRequest {}

#[derive(CandidType, Clone, Debug, Default, Serialize, Deserialize)]
struct CreateBatchResponse {
    batch_id: u128,
}

#[derive(CandidType, Clone, Debug, Default, Serialize, Deserialize)]
struct CreateChunkRequest {
    batch_id: u128,
    content: Vec<u8>,
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
    chunk_ids: Vec<String>,
}

fn make_chunked_asset(agent: &Agent, asset_location: AssetLocation) -> DfxResult<ChunkedAsset> {
    Ok(ChunkedAsset {
        asset_location: asset_location,
        chunk_ids: vec![],
    })
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

    let create_batch_args = CreateBatchRequest {};
    let response = agent
        .update(&canister_id, CREATE_BATCH)
        .with_arg(candid::Encode!(&create_batch_args)?)
        .expire_after(timeout)
        .call_and_wait(waiter_with_timeout(timeout))
        .await?;
    let create_batch_response = candid::Decode!(&response, CreateBatchResponse)?;
    let batch_id = create_batch_response.batch_id;

    let chunked_assets: DfxResult<Vec<ChunkedAsset>> = asset_locations
        .into_iter()
        .map(|loc| make_chunked_asset(agent, loc))
        .collect();
    let chunked_assets = chunked_assets?;

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
