use crate::lib::canister_info::assets::AssetsCanisterInfo;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::waiter::waiter_with_timeout;
use candid::{CandidType, Decode, Encode, Nat};

use anyhow::anyhow;
use delay::{Delay, Waiter};
use flate2::write::GzEncoder;
use flate2::Compression;
use ic_agent::Agent;
use ic_types::Principal;
use mime::Mime;
use openssl::sha::Sha256;
use serde::Deserialize;
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;
use walkdir::WalkDir;
use crate::lib::installers::assets::content_encoders::ContentEncoder;

mod content_encoders;

const CREATE_BATCH: &str = "create_batch";
const CREATE_CHUNK: &str = "create_chunk";
const COMMIT_BATCH: &str = "commit_batch";
const LIST: &str = "list";
const MAX_CHUNK_SIZE: usize = 1_900_000;
const CONTENT_ENCODING_GZIP: &str = "gzip";

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

async fn make_project_asset_encoding(
    canister_call_params: &CanisterCallParams<'_>,
    batch_id: &Nat,
    asset_location: &AssetLocation,
    container_assets: &HashMap<String, AssetDetails>,
    content: &[u8],
    content_encoding: &str,
    media_type: &Mime,
) -> DfxResult<ProjectAssetEncoding> {
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

    let chunk_ids = if already_in_place {
        println!(
            "  {} ({} bytes) sha {} is already installed",
            &asset_location.key,
            content.len(),
            hex::encode(&sha256),
        );
        vec![]
    } else {
        upload_content_chunks(canister_call_params, batch_id, &asset_location, content).await?
    };

    Ok(ProjectAssetEncoding {
        chunk_ids,
        sha256,
        already_in_place,
    })
}

async fn make_project_asset(
    canister_call_params: &CanisterCallParams<'_>,
    batch_id: &Nat,
    asset_location: AssetLocation,
    container_assets: &HashMap<String, AssetDetails>,
) -> DfxResult<ProjectAsset> {
    let content = std::fs::read(&asset_location.source)?;

    let media_type = mime_guess::from_path(&asset_location.source)
        .first()
        .unwrap_or(mime::APPLICATION_OCTET_STREAM);

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

    for content_encoding in content_encodings(&media_type) {
        add_encoding(
            &mut encodings,
            &content_encoding,
            canister_call_params,
            batch_id,
            &asset_location,
            container_assets,
            &content,
            &media_type,
        )
        .await?;
    }

    Ok(ProjectAsset {
        asset_location,
        media_type,
        encodings,
    })
}

fn content_encodings(media_type: &Mime) -> Vec<&str> {
    match media_type.subtype() {
        mime::JAVASCRIPT => vec![CONTENT_ENCODING_GZIP],
        _ => vec![],
    }
}

fn applicable_encoders(media_type: &Mime) -> Vec<impl ContentEncoder> {
    match media_type.subtype() {
        //mime::JAVASCRIPT => vec![CONTENT_ENCODING_GZIP],
        _ => vec![],
    }

}

async fn add_identity_encoding(
    encodings: &mut HashMap<String, ProjectAssetEncoding>,
    canister_call_params: &CanisterCallParams<'_>,
    batch_id: &Nat,
    asset_location: &AssetLocation,
    container_assets: &HashMap<String, AssetDetails>,
    content: &[u8],
    media_type: &Mime,
) -> DfxResult {
    let content_encoding = "identity".to_string();
    let project_asset_encoding = make_project_asset_encoding(
        canister_call_params,
        batch_id,
        &asset_location,
        container_assets,
        &content,
        &content_encoding,
        media_type,
    )
    .await?;

    encodings.insert(content_encoding, project_asset_encoding);
    Ok(())
}

async fn add_encoding(
    encodings: &mut HashMap<String, ProjectAssetEncoding>,
    content_encoding: &str,
    canister_call_params: &CanisterCallParams<'_>,
    batch_id: &Nat,
    asset_location: &AssetLocation,
    container_assets: &HashMap<String, AssetDetails>,
    content: &[u8],
    media_type: &Mime,
) -> DfxResult {
    let encoded_content = encode(content_encoding, content)?;
    let project_asset_encoding = make_project_asset_encoding(
        canister_call_params,
        batch_id,
        &asset_location,
        container_assets,
        &encoded_content,
        &content_encoding.to_string(),
        media_type,
    )
    .await?;

    encodings.insert(content_encoding.to_string(), project_asset_encoding);
    Ok(())
}

fn encode(content_encoding: &str, content: &[u8]) -> DfxResult<Vec<u8>> {
    match content_encoding {
        CONTENT_ENCODING_GZIP => encode_gzip(content),
        _ => Err(anyhow!(format!(
            "Unsupported content encoding {}",
            content_encoding
        ))),
    }
}

fn encode_gzip(content: &[u8]) -> DfxResult<Vec<u8>> {
    let mut e = GzEncoder::new(Vec::new(), Compression::default());
    e.write(content).unwrap();
    let x = e.finish()?;
    Ok(x)
}

async fn make_project_assets(
    canister_call_params: &CanisterCallParams<'_>,
    batch_id: &Nat,
    locs: Vec<AssetLocation>,
    container_assets: &HashMap<String, AssetDetails>,
) -> DfxResult<HashMap<String, ProjectAsset>> {
    let mut project_assets = HashMap::new();
    for loc in locs {
        let project_asset =
            make_project_asset(canister_call_params, batch_id, loc, &container_assets).await?;
        project_assets.insert(project_asset.asset_location.key.clone(), project_asset);
    }
    Ok(project_assets)
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

    let project_assets = make_project_assets(
        &canister_call_params,
        &batch_id,
        asset_locations,
        &container_assets,
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
