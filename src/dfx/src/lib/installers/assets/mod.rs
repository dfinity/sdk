use crate::lib::canister_info::assets::AssetsCanisterInfo;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::installers::assets::content::Content;
use crate::lib::installers::assets::content_encoder::ContentEncoder;
use crate::lib::waiter::waiter_with_timeout;
use candid::{CandidType, Decode, Encode, Nat};

use futures::future::try_join_all;
use futures::TryFutureExt;
use futures_intrusive::sync::SharedSemaphore;
use garcon::{Delay, Waiter};
use ic_agent::Agent;
use ic_types::Principal;
use mime::Mime;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use walkdir::WalkDir;

mod content;
mod content_encoder;

const CONTENT_ENCODING_IDENTITY: &str = "identity";
const CREATE_BATCH: &str = "create_batch";
const CREATE_CHUNK: &str = "create_chunk";
const COMMIT_BATCH: &str = "commit_batch";
const LIST: &str = "list";
const MAX_CHUNK_SIZE: usize = 1_900_000;

// Maximum MB of file data to load at once.  More memory may be used, due to encodings.
const MAX_SIMULTANEOUS_LOADED_MB: usize = 50;

// The most mb any one file is considered to have for purposes of limiting data loaded at once.
// Any file counts as at least 1 mb.
const MAX_COST_SINGLE_FILE_MB: usize = 45;

// How many simultaneous Agent.call() to create_chunk
const MAX_SIMULTANEOUS_CREATE_CHUNK_CALLS: usize = 1;

// How many simultaneous Agent.wait() on create_chunk result
const MAX_SIMULTANEOUS_CREATE_CHUNK_WAITS: usize = 1;

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

struct ProjectAssetEncoding {
    chunk_ids: Vec<Nat>,
    sha256: Vec<u8>,
    already_in_place: bool,
}

struct ProjectAsset {
    asset_location: AssetLocation,
    media_type: Mime,
    encodings: HashMap<String, ProjectAssetEncoding>,
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
    create_chunk_call_semaphore: &SharedSemaphore,
    create_chunk_wait_semaphore: &SharedSemaphore,
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
        let mut builder = canister_call_params
            .agent
            .update(&canister_call_params.canister_id, CREATE_CHUNK);
        let builder = builder
            .with_arg(&args)
            .expire_after(canister_call_params.timeout);
        let request_id_result = {
            let _releaser = create_chunk_call_semaphore.acquire(1).await;
            builder.call().await
        };
        let wait_result = match request_id_result {
            Ok(request_id) => {
                let _releaser = create_chunk_wait_semaphore.acquire(1).await;
                canister_call_params
                    .agent
                    .wait(
                        request_id,
                        &canister_call_params.canister_id,
                        waiter_with_timeout(canister_call_params.timeout),
                    )
                    .await
            }
            Err(err) => Err(err),
        };
        match wait_result.map_err(DfxError::from).and_then(|response| {
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
    content: &Content,
    content_encoding: &str,
    create_chunk_call_semaphore: &SharedSemaphore,
    create_chunk_wait_semaphore: &SharedSemaphore,
) -> DfxResult<Vec<Nat>> {
    if content.data.is_empty() {
        let empty = vec![];
        let chunk_id = create_chunk(
            canister_call_params,
            batch_id,
            &empty,
            create_chunk_call_semaphore,
            create_chunk_wait_semaphore,
        )
        .await?;
        println!(
            "  {}{} 1/1 (0 bytes)",
            &asset_location.key,
            content_encoding_descriptive_suffix(content_encoding)
        );
        return Ok(vec![chunk_id]);
    }

    let count = (content.data.len() + MAX_CHUNK_SIZE - 1) / MAX_CHUNK_SIZE;
    let chunks_futures: Vec<_> = content
        .data
        .chunks(MAX_CHUNK_SIZE)
        .enumerate()
        .map(|(i, data_chunk)| {
            create_chunk(
                canister_call_params,
                batch_id,
                data_chunk,
                create_chunk_call_semaphore,
                create_chunk_wait_semaphore,
            )
            .map_ok(move |chunk_id| {
                println!(
                    "  {}{} {}/{} ({} bytes)",
                    &asset_location.key,
                    content_encoding_descriptive_suffix(content_encoding),
                    i + 1,
                    count,
                    data_chunk.len(),
                );
                chunk_id
            })
        })
        .collect();
    try_join_all(chunks_futures).await
}

#[allow(clippy::too_many_arguments)]
async fn make_project_asset_encoding(
    canister_call_params: &CanisterCallParams<'_>,
    batch_id: &Nat,
    asset_location: &AssetLocation,
    container_assets: &HashMap<String, AssetDetails>,
    content: &Content,
    content_encoding: &str,
    create_chunk_call_semaphore: &SharedSemaphore,
    create_chunk_wait_semaphore: &SharedSemaphore,
) -> DfxResult<ProjectAssetEncoding> {
    let sha256 = content.sha256();

    let already_in_place = if let Some(container_asset) = container_assets.get(&asset_location.key)
    {
        if container_asset.content_type != content.media_type.to_string() {
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

    let chunk_ids = if already_in_place {
        println!(
            "  {}{} ({} bytes) sha {} is already installed",
            &asset_location.key,
            content_encoding_descriptive_suffix(content_encoding),
            content.data.len(),
            hex::encode(&sha256),
        );
        vec![]
    } else {
        upload_content_chunks(
            canister_call_params,
            batch_id,
            &asset_location,
            content,
            content_encoding,
            create_chunk_call_semaphore,
            create_chunk_wait_semaphore,
        )
        .await?
    };

    Ok(ProjectAssetEncoding {
        chunk_ids,
        sha256,
        already_in_place,
    })
}

fn content_encoding_descriptive_suffix(content_encoding: &str) -> String {
    if content_encoding == CONTENT_ENCODING_IDENTITY {
        "".to_string()
    } else {
        format!(" ({})", content_encoding)
    }
}

async fn make_project_asset(
    canister_call_params: &CanisterCallParams<'_>,
    batch_id: &Nat,
    asset_location: AssetLocation,
    container_assets: &HashMap<String, AssetDetails>,
    file_semaphore: &SharedSemaphore,
    create_chunk_call_semaphore: &SharedSemaphore,
    create_chunk_wait_semaphore: &SharedSemaphore,
) -> DfxResult<ProjectAsset> {
    let file_size = std::fs::metadata(&asset_location.source)?.len();
    let permits = std::cmp::max(
        1,
        std::cmp::min(
            ((file_size + 999999) / 1000000) as usize,
            MAX_COST_SINGLE_FILE_MB,
        ),
    );
    let _releaser = file_semaphore.acquire(permits).await;
    let content = Content::load(&asset_location.source)?;

    let encodings = make_encodings(
        canister_call_params,
        batch_id,
        &asset_location,
        container_assets,
        &content,
        create_chunk_call_semaphore,
        create_chunk_wait_semaphore,
    )
    .await?;

    Ok(ProjectAsset {
        asset_location,
        media_type: content.media_type,
        encodings,
    })
}

// todo: make this configurable https://github.com/dfinity/dx-triage/issues/152
fn applicable_encoders(media_type: &Mime) -> Vec<ContentEncoder> {
    match (media_type.type_(), media_type.subtype()) {
        (mime::TEXT, _) | (_, mime::JAVASCRIPT) | (_, mime::HTML) => vec![ContentEncoder::Gzip],
        _ => vec![],
    }
}

#[allow(clippy::too_many_arguments)]
async fn make_encoding(
    canister_call_params: &CanisterCallParams<'_>,
    batch_id: &Nat,
    asset_location: &AssetLocation,
    container_assets: &HashMap<String, AssetDetails>,
    content: &Content,
    encoder: &Option<ContentEncoder>,
    create_chunk_call_semaphore: &SharedSemaphore,
    create_chunk_wait_semaphore: &SharedSemaphore,
) -> DfxResult<Option<(String, ProjectAssetEncoding)>> {
    match encoder {
        None => {
            let identity_asset_encoding = make_project_asset_encoding(
                canister_call_params,
                batch_id,
                &asset_location,
                container_assets,
                &content,
                CONTENT_ENCODING_IDENTITY,
                create_chunk_call_semaphore,
                create_chunk_wait_semaphore,
            )
            .await?;
            Ok(Some((
                CONTENT_ENCODING_IDENTITY.to_string(),
                identity_asset_encoding,
            )))
        }
        Some(encoder) => {
            let encoded = content.encode(&encoder)?;
            if encoded.data.len() < content.data.len() {
                let content_encoding = format!("{}", encoder);
                let project_asset_encoding = make_project_asset_encoding(
                    canister_call_params,
                    batch_id,
                    &asset_location,
                    container_assets,
                    &encoded,
                    &content_encoding,
                    create_chunk_call_semaphore,
                    create_chunk_wait_semaphore,
                )
                .await?;
                Ok(Some((content_encoding, project_asset_encoding)))
            } else {
                Ok(None)
            }
        }
    }
}

async fn make_encodings(
    canister_call_params: &CanisterCallParams<'_>,
    batch_id: &Nat,
    asset_location: &AssetLocation,
    container_assets: &HashMap<String, AssetDetails>,
    content: &Content,
    create_chunk_call_semaphore: &SharedSemaphore,
    create_chunk_wait_semaphore: &SharedSemaphore,
) -> DfxResult<HashMap<String, ProjectAssetEncoding>> {
    let mut encoders = vec![None];
    for encoder in applicable_encoders(&content.media_type) {
        encoders.push(Some(encoder));
    }

    let encoding_futures: Vec<_> = encoders
        .iter()
        .map(|maybe_encoder| {
            make_encoding(
                canister_call_params,
                batch_id,
                asset_location,
                container_assets,
                content,
                maybe_encoder,
                create_chunk_call_semaphore,
                create_chunk_wait_semaphore,
            )
        })
        .collect();

    let encodings = try_join_all(encoding_futures).await?;

    let mut result: HashMap<String, ProjectAssetEncoding> = HashMap::new();

    for (key, value) in encodings.into_iter().flatten() {
        result.insert(key, value);
    }
    Ok(result)
}

async fn make_project_assets(
    canister_call_params: &CanisterCallParams<'_>,
    batch_id: &Nat,
    locs: Vec<AssetLocation>,
    container_assets: &HashMap<String, AssetDetails>,
    file_semaphore: &SharedSemaphore,
    create_chunk_call_semaphore: &SharedSemaphore,
    create_chunk_wait_semaphore: &SharedSemaphore,
) -> DfxResult<HashMap<String, ProjectAsset>> {
    let project_asset_futures: Vec<_> = locs
        .iter()
        .map(|loc| {
            make_project_asset(
                canister_call_params,
                batch_id,
                loc.clone(),
                &container_assets,
                file_semaphore,
                create_chunk_call_semaphore,
                create_chunk_wait_semaphore,
            )
        })
        .collect();
    let project_assets = try_join_all(project_asset_futures).await?;

    let mut hm = HashMap::new();
    for project_asset in project_assets {
        hm.insert(project_asset.asset_location.key.clone(), project_asset);
    }
    Ok(hm)
}

async fn commit_batch(
    canister_call_params: &CanisterCallParams<'_>,
    batch_id: &Nat,
    project_assets: HashMap<String, ProjectAsset>,
    container_assets: HashMap<String, AssetDetails>,
) -> DfxResult {
    let mut container_assets = container_assets;

    let mut operations = vec![];

    delete_obsolete_assets(&mut operations, &project_assets, &mut container_assets);
    create_new_assets(&mut operations, &project_assets, &container_assets);
    unset_obsolete_encodings(&mut operations, &project_assets, &container_assets);
    set_encodings(&mut operations, project_assets);

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
    project_assets: &HashMap<String, ProjectAsset>,
    container_assets: &mut HashMap<String, AssetDetails>,
) {
    let mut deleted_container_assets = vec![];
    for (key, container_asset) in container_assets.iter() {
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
}

fn create_new_assets(
    operations: &mut Vec<BatchOperationKind>,
    project_assets: &HashMap<String, ProjectAsset>,
    container_assets: &HashMap<String, AssetDetails>,
) {
    for (key, project_asset) in project_assets {
        if !container_assets.contains_key(key) {
            operations.push(BatchOperationKind::CreateAsset(CreateAssetArguments {
                key: key.clone(),
                content_type: project_asset.media_type.to_string(),
            }));
        }
    }
}

fn unset_obsolete_encodings(
    operations: &mut Vec<BatchOperationKind>,
    project_assets: &HashMap<String, ProjectAsset>,
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
    project_assets: HashMap<String, ProjectAsset>,
) {
    for (key, project_asset) in project_assets {
        for (content_encoding, v) in project_asset.encodings {
            if v.already_in_place {
                continue;
            }

            operations.push(BatchOperationKind::SetAssetContent(
                SetAssetContentArguments {
                    key: key.clone(),
                    content_encoding,
                    chunk_ids: v.chunk_ids,
                    sha256: Some(v.sha256),
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

    // The "file" semaphore limits how much file data to load at once.  A given loaded file's data
    // may be simultaneously encoded (gzip and so forth).
    let file_semaphore = SharedSemaphore::new(true, MAX_SIMULTANEOUS_LOADED_MB);

    // The create_chunk call semaphore limits the number of simultaneous
    // agent.call()s to create_chunk.
    let create_chunk_call_semaphore =
        SharedSemaphore::new(true, MAX_SIMULTANEOUS_CREATE_CHUNK_CALLS);

    // The create_chunk wait semaphore limits the number of simultaneous
    // agent.wait() calls for outstanding create_chunk requests.
    let create_chunk_wait_semaphore =
        SharedSemaphore::new(true, MAX_SIMULTANEOUS_CREATE_CHUNK_WAITS);

    let project_assets = make_project_assets(
        &canister_call_params,
        &batch_id,
        asset_locations,
        &container_assets,
        &file_semaphore,
        &create_chunk_call_semaphore,
        &create_chunk_wait_semaphore,
    )
    .await?;

    commit_batch(
        &canister_call_params,
        &batch_id,
        project_assets,
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
