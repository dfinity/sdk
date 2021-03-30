use crate::lib::canister_info::assets::AssetsCanisterInfo;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::waiter::waiter_with_timeout;
use candid::{CandidType, Decode, Encode, Nat};

use anyhow::anyhow;
use delay::{Delay, Waiter};
use ic_agent::Agent;
use ic_types::Principal;
use mime::Mime;
use openssl::sha::Sha256;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use walkdir::WalkDir;

const CREATE_BATCH: &str = "create_batch";
const CREATE_CHUNK: &str = "create_chunk";
const COMMIT_BATCH: &str = "commit_batch";
const LIST: &str = "list";
const MAX_CHUNK_SIZE: usize = 1_900_000;

#[derive(CandidType, Debug)]
struct CreateBatchRequest {}

#[derive(CandidType, Debug, Deserialize)]
struct CreateBatchResponse {
    batch_id: Nat,
}

#[derive(CandidType, Debug, Deserialize)]
struct CreateChunkRequest<'a> {
    batch_id: Nat,
    #[serde(with = "serde_bytes")]
    content: &'a [u8],
}

#[derive(CandidType, Debug, Deserialize)]
struct CreateChunkResponse {
    chunk_id: Nat,
}

#[derive(CandidType, Debug)]
struct GetRequest {
    key: String,
    accept_encodings: Vec<String>,
}

#[derive(CandidType, Debug, Deserialize)]
struct GetResponse {
    #[serde(with = "serde_bytes")]
    contents: Vec<u8>,
    content_type: String,
    content_encoding: String,
}

#[derive(CandidType, Debug)]
struct ListAssetsRequest {}

#[derive(CandidType, Debug, Deserialize)]
struct AssetEncodingDetails {
    content_encoding: String,
    sha256: Option<Vec<u8>>,
}

#[derive(CandidType, Debug, Deserialize)]
struct AssetDetails {
    key: String,
    encodings: Vec<AssetEncodingDetails>,
    content_type: String,
}

#[derive(CandidType, Debug)]
struct CreateAssetArguments {
    key: String,
    content_type: String,
}
#[derive(CandidType, Debug)]
struct SetAssetContentArguments {
    key: String,
    content_encoding: String,
    chunk_ids: Vec<Nat>,
    sha256: Option<Vec<u8>>,
}
#[derive(CandidType, Debug)]
struct UnsetAssetContentArguments {
    key: String,
    content_encoding: String,
}
#[derive(CandidType, Debug)]
struct DeleteAssetArguments {
    key: String,
}
#[derive(CandidType, Debug)]
struct ClearArguments {}

#[derive(CandidType, Debug)]
enum BatchOperationKind {
    CreateAsset(CreateAssetArguments),

    SetAssetContent(SetAssetContentArguments),

    UnsetAssetContent(UnsetAssetContentArguments),

    DeleteAsset(DeleteAssetArguments),

    _Clear(ClearArguments),
}

#[derive(CandidType, Debug)]
struct CommitBatchArguments<'a> {
    batch_id: &'a Nat,
    operations: Vec<BatchOperationKind>,
}

#[derive(Clone, Debug)]
struct AssetLocation {
    source: PathBuf,
    key: String,
}

struct ChunkedAssetEncoding {
    chunk_ids: Vec<Nat>,
    sha256: Vec<u8>,
    in_sync: bool,
}

struct ChunkedAsset {
    asset_location: AssetLocation,
    media_type: Mime,
    encodings: HashMap<String, ChunkedAssetEncoding>,
}

struct CanisterCallParams<'a> {
    agent: &'a Agent,
    canister_id: Principal,
    timeout: Duration,
}

async fn create_chunk(
    canister_call_params: &CanisterCallParams<'_>,
    batch_id: &Nat,
    content: &[u8],
) -> DfxResult<Nat> {
    let batch_id = batch_id.clone();
    let args = CreateChunkRequest { batch_id, content };
    let args = candid::Encode!(&args)?;

    let mut waiter = Delay::builder()
        .timeout(std::time::Duration::from_secs(30))
        .throttle(std::time::Duration::from_secs(1))
        .build();
    waiter.start();

    loop {
        match canister_call_params
            .agent
            .update(&canister_call_params.canister_id, CREATE_CHUNK)
            .with_arg(&args)
            .expire_after(canister_call_params.timeout)
            .call_and_wait(waiter_with_timeout(canister_call_params.timeout))
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

async fn upload_content_chunks(
    canister_call_params: &CanisterCallParams<'_>,
    batch_id: &Nat,
    asset_location: &AssetLocation,
    content: &[u8],
) -> DfxResult<Vec<Nat>> {
    let mut chunk_ids: Vec<Nat> = vec![];
    let chunks = content.chunks(MAX_CHUNK_SIZE);
    let (num_chunks, _) = chunks.size_hint();
    for (i, data_chunk) in chunks.enumerate() {
        println!(
            "  {} {}/{} ({} bytes)",
            &asset_location.key,
            i + 1,
            num_chunks,
            data_chunk.len()
        );
        chunk_ids.push(create_chunk(canister_call_params, batch_id, data_chunk).await?);
    }
    if chunk_ids.is_empty() {
        println!("  {} 1/1 (0 bytes)", &asset_location.key);
        let empty = vec![];
        chunk_ids.push(create_chunk(canister_call_params, batch_id, &empty).await?);
    }
    Ok(chunk_ids)
}

async fn make_chunked_asset_encoding(
    canister_call_params: &CanisterCallParams<'_>,
    batch_id: &Nat,
    asset_location: &AssetLocation,
    container_assets: &HashMap<String, AssetDetails>,
    content: &[u8],
    content_encoding: &str,
    media_type: &Mime,
) -> DfxResult<ChunkedAssetEncoding> {
    let mut sha256 = Sha256::new();
    sha256.update(&content);
    let sha256 = sha256.finish().to_vec();

    let already_in_place = if let Some(container_asset) = container_assets.get(&asset_location.key)
    {
        if container_asset.content_type != media_type.to_string() {
            false
        } else if let Some(container_asset_encoding_sha256) = container_asset
            .encodings
            .iter()
            .find(|details| details.content_encoding == content_encoding)
            .and_then(|details| details.sha256.as_ref())
        {
            container_asset_encoding_sha256 == &sha256
        } else {
            false
        }
    } else {
        false
    };

    if already_in_place {
        println!(
            "  {} ({} bytes) sha {} is already installed",
            &asset_location.key,
            content.len(),
            hex::encode(&sha256),
        );
        Ok(ChunkedAssetEncoding {
            chunk_ids: vec![],
            sha256,
            in_sync: true,
        })
    } else {
        let chunk_ids =
            upload_content_chunks(canister_call_params, batch_id, &asset_location, content).await?;
        Ok(ChunkedAssetEncoding {
            chunk_ids,
            sha256,
            in_sync: false,
        })
    }
}

async fn make_chunked_asset(
    canister_call_params: &CanisterCallParams<'_>,
    batch_id: &Nat,
    asset_location: AssetLocation,
    container_assets: &HashMap<String, AssetDetails>,
) -> DfxResult<ChunkedAsset> {
    let content = std::fs::read(&asset_location.source)?;

    let media_type = mime_guess::from_path(&asset_location.source)
        .first()
        .ok_or_else(|| {
            anyhow!(
                "Unable to determine content type for '{}'.",
                asset_location.source.to_string_lossy()
            )
        })?;

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

    let mut encodings = HashMap::new();

    add_identity_encoding(
        &mut encodings,
        canister_call_params,
        batch_id,
        &asset_location,
        container_assets,
        &content,
        &media_type,
    )
    .await?;

    Ok(ChunkedAsset {
        asset_location,
        media_type,
        encodings,
    })
}

async fn add_identity_encoding(
    encodings: &mut HashMap<String, ChunkedAssetEncoding>,
    canister_call_params: &CanisterCallParams<'_>,
    batch_id: &Nat,
    asset_location: &AssetLocation,
    container_assets: &HashMap<String, AssetDetails>,
    content: &[u8],
    media_type: &Mime,
) -> DfxResult {
    let content_encoding = "identity".to_string();
    let chunked_asset_encoding = make_chunked_asset_encoding(
        canister_call_params,
        batch_id,
        &asset_location,
        container_assets,
        &content,
        &content_encoding,
        media_type,
    )
    .await?;

    encodings.insert(content_encoding, chunked_asset_encoding);
    Ok(())
}

async fn make_chunked_assets(
    canister_call_params: &CanisterCallParams<'_>,
    batch_id: &Nat,
    locs: Vec<AssetLocation>,
    container_assets: &HashMap<String, AssetDetails>,
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
        chunked_assets.push(
            make_chunked_asset(canister_call_params, batch_id, loc, &container_assets).await?,
        );
    }
    Ok(chunked_assets)
}

async fn commit_batch(
    canister_call_params: &CanisterCallParams<'_>,
    batch_id: &Nat,
    project_assets: Vec<ChunkedAsset>,
    container_assets: HashMap<String, AssetDetails>,
) -> DfxResult {
    let mut container_assets = container_assets;
    let project_assets: HashMap<_, _> = project_assets
        .iter()
        .map(|e| (e.asset_location.key.clone(), e))
        .collect();

    let mut operations = vec![];

    container_assets = delete_obsolete_assets(&mut operations, &project_assets, container_assets);
    create_new_assets(&mut operations, &project_assets, &container_assets);
    unset_obsolete_encodings(&mut operations, &project_assets, &container_assets);
    set_encodings(&mut operations, &project_assets);

    let arg = CommitBatchArguments {
        batch_id,
        operations,
    };
    let arg = candid::Encode!(&arg)?;
    canister_call_params
        .agent
        .update(&canister_call_params.canister_id, COMMIT_BATCH)
        .with_arg(arg)
        .expire_after(canister_call_params.timeout)
        .call_and_wait(waiter_with_timeout(canister_call_params.timeout))
        .await?;
    Ok(())
}

fn delete_obsolete_assets(
    operations: &mut Vec<BatchOperationKind>,
    project_assets: &HashMap<String, &ChunkedAsset>,
    container_assets: HashMap<String, AssetDetails>,
) -> HashMap<String, AssetDetails> {
    let mut container_assets = container_assets;
    let mut deleted_container_assets = vec![];
    for (key, container_asset) in &container_assets {
        let project_asset = project_assets.get(key);
        if project_asset
            .filter(|&x| x.media_type.to_string() == container_asset.content_type)
            .is_none()
        {
            operations.push(BatchOperationKind::DeleteAsset(DeleteAssetArguments {
                key: key.clone(),
            }));
            deleted_container_assets.push(key.clone());
        }
    }
    for k in deleted_container_assets {
        container_assets.remove(&k);
    }
    container_assets
}

fn create_new_assets(
    operations: &mut Vec<BatchOperationKind>,
    project_assets: &HashMap<String, &ChunkedAsset>,
    container_assets: &HashMap<String, AssetDetails>,
) {
    for (key, &chunked_asset) in project_assets {
        if !container_assets.contains_key(key) {
            operations.push(BatchOperationKind::CreateAsset(CreateAssetArguments {
                key: key.clone(),
                content_type: chunked_asset.media_type.to_string(),
            }));
        }
    }
}

fn unset_obsolete_encodings(
    operations: &mut Vec<BatchOperationKind>,
    project_assets: &HashMap<String, &ChunkedAsset>,
    container_assets: &HashMap<String, AssetDetails>,
) {
    for (key, details) in container_assets {
        let project_asset = project_assets.get(key);
        for encoding_details in &details.encodings {
            let project_contains_encoding = project_asset
                .filter(|project_asset| {
                    project_asset
                        .encodings
                        .contains_key(&encoding_details.content_encoding)
                })
                .is_some();
            if !project_contains_encoding {
                operations.push(BatchOperationKind::UnsetAssetContent(
                    UnsetAssetContentArguments {
                        key: key.clone(),
                        content_encoding: encoding_details.content_encoding.clone(),
                    },
                ));
            }
        }
    }
}

fn set_encodings(
    operations: &mut Vec<BatchOperationKind>,
    project_assets: &HashMap<String, &ChunkedAsset>,
) {
    for (key, chunked_asset) in project_assets {
        for (content_encoding, v) in &chunked_asset.encodings {
            if v.in_sync {
                continue;
            }

            operations.push(BatchOperationKind::SetAssetContent(
                SetAssetContentArguments {
                    key: key.clone(),
                    content_encoding: content_encoding.clone(),
                    chunk_ids: v.chunk_ids.clone(),
                    sha256: Some(v.sha256.clone()),
                },
            ));
        }
    }
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
                    .expect("cannot strip prefix");
                let key = String::from("/") + relative.to_string_lossy().as_ref();

                AssetLocation { source, key }
            })
        })
        .collect();

    let canister_id = info.get_canister_id().expect("Could not find canister ID.");
    let canister_call_params = CanisterCallParams {
        agent,
        canister_id,
        timeout,
    };

    let container_assets = list_assets(&canister_call_params).await?;

    let batch_id = create_batch(&canister_call_params).await?;

    let chunked_assets = make_chunked_assets(
        &canister_call_params,
        &batch_id,
        asset_locations,
        &container_assets,
    )
    .await?;

    commit_batch(
        &canister_call_params,
        &batch_id,
        chunked_assets,
        container_assets,
    )
    .await?;

    Ok(())
}

async fn create_batch(canister_call_params: &CanisterCallParams<'_>) -> DfxResult<Nat> {
    let create_batch_args = CreateBatchRequest {};
    let response = canister_call_params
        .agent
        .update(&canister_call_params.canister_id, CREATE_BATCH)
        .with_arg(candid::Encode!(&create_batch_args)?)
        .expire_after(canister_call_params.timeout)
        .call_and_wait(waiter_with_timeout(canister_call_params.timeout))
        .await?;
    let create_batch_response = candid::Decode!(&response, CreateBatchResponse)?;
    Ok(create_batch_response.batch_id)
}

async fn list_assets(
    canister_call_params: &CanisterCallParams<'_>,
) -> DfxResult<HashMap<String, AssetDetails>> {
    let args = ListAssetsRequest {};
    let response = canister_call_params
        .agent
        .update(&canister_call_params.canister_id, LIST)
        .with_arg(candid::Encode!(&args)?)
        .expire_after(canister_call_params.timeout)
        .call_and_wait(waiter_with_timeout(canister_call_params.timeout))
        .await?;

    let assets: HashMap<_, _> = candid::Decode!(&response, Vec<AssetDetails>)?
        .into_iter()
        .map(|d| (d.key.clone(), d))
        .collect();

    Ok(assets)
}
