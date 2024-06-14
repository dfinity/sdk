use crate::asset::config::AssetConfig;
use crate::asset::content::Content;
use crate::asset::content_encoder::ContentEncoder;
use crate::batch_upload::semaphores::Semaphores;
use crate::canister_api::methods::chunk::create_chunk;
use crate::canister_api::types::asset::AssetDetails;
use crate::error::CreateChunkError;
use crate::error::CreateEncodingError;
use crate::error::CreateEncodingError::EncodeContentFailed;
use crate::error::CreateProjectAssetError;
use candid::Nat;
use futures::future::try_join_all;
use futures::TryFutureExt;
use ic_utils::Canister;
use mime::Mime;
use slog::{debug, info, Logger};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

const CONTENT_ENCODING_IDENTITY: &str = "identity";

// The most mb any one file is considered to have for purposes of limiting data loaded at once.
// Any file counts as at least 1 mb.
const MAX_COST_SINGLE_FILE_MB: usize = 45;

const MAX_CHUNK_SIZE: usize = 1_900_000;

#[derive(Clone, Debug)]
pub(crate) struct AssetDescriptor {
    pub(crate) source: PathBuf,
    pub(crate) key: String,
    pub(crate) config: AssetConfig,
}

pub(crate) struct ProjectAssetEncoding {
    pub(crate) chunk_ids: Vec<Nat>,
    pub(crate) sha256: Vec<u8>,
    pub(crate) already_in_place: bool,
}

pub(crate) struct ProjectAsset {
    pub(crate) asset_descriptor: AssetDescriptor,
    pub(crate) media_type: Mime,
    pub(crate) encodings: HashMap<String, ProjectAssetEncoding>,
}

pub(crate) struct ChunkUploader<'agent> {
    canister: Canister<'agent>,
    batch_id: Nat,
    chunks: Arc<AtomicUsize>,
    bytes: Arc<AtomicUsize>,
}
impl<'agent> ChunkUploader<'agent> {
    pub(crate) fn new(canister: Canister<'agent>, batch_id: Nat) -> Self {
        Self {
            canister,
            batch_id,
            chunks: Arc::new(AtomicUsize::new(0)),
            bytes: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub(crate) async fn create_chunk(
        &self,
        contents: &[u8],
        semaphores: &Semaphores,
    ) -> Result<Nat, CreateChunkError> {
        self.chunks.fetch_add(1, Ordering::SeqCst);
        self.bytes.fetch_add(contents.len(), Ordering::SeqCst);
        create_chunk(&self.canister, &self.batch_id, contents, semaphores).await
    }

    pub(crate) fn bytes(&self) -> usize {
        self.bytes.load(Ordering::SeqCst)
    }
    pub(crate) fn chunks(&self) -> usize {
        self.chunks.load(Ordering::SeqCst)
    }
}

#[allow(clippy::too_many_arguments)]
async fn make_project_asset_encoding(
    chunk_upload_target: Option<&ChunkUploader<'_>>,
    asset_descriptor: &AssetDescriptor,
    canister_assets: &HashMap<String, AssetDetails>,
    content: &Content,
    content_encoding: &str,
    semaphores: &Semaphores,
    logger: &Logger,
) -> Result<ProjectAssetEncoding, CreateChunkError> {
    let sha256 = content.sha256();

    let already_in_place = if let Some(canister_asset) = canister_assets.get(&asset_descriptor.key)
    {
        if canister_asset.content_type != content.media_type.to_string() {
            false
        } else if let Some(canister_asset_encoding_sha256) = canister_asset
            .encodings
            .iter()
            .find(|details| details.content_encoding == content_encoding)
            .and_then(|details| details.sha256.as_ref())
        {
            canister_asset_encoding_sha256 == &sha256
        } else {
            false
        }
    } else {
        false
    };

    let chunk_ids = if already_in_place {
        info!(
            logger,
            "  {}{} ({} bytes) sha {} is already installed",
            &asset_descriptor.key,
            content_encoding_descriptive_suffix(content_encoding),
            content.data.len(),
            hex::encode(&sha256),
        );
        vec![]
    } else if let Some(target) = chunk_upload_target {
        upload_content_chunks(
            target,
            asset_descriptor,
            content,
            &sha256,
            content_encoding,
            semaphores,
            logger,
        )
        .await?
    } else {
        info!(
            logger,
            "  {}{} ({} bytes) sha {} will be uploaded",
            &asset_descriptor.key,
            content_encoding_descriptive_suffix(content_encoding),
            content.data.len(),
            hex::encode(&sha256),
        );
        vec![]
    };

    Ok(ProjectAssetEncoding {
        chunk_ids,
        sha256,
        already_in_place,
    })
}

#[allow(clippy::too_many_arguments)]
async fn make_encoding(
    chunk_upload_target: Option<&ChunkUploader<'_>>,
    asset_descriptor: &AssetDescriptor,
    canister_assets: &HashMap<String, AssetDetails>,
    content: &Content,
    encoder: &Option<ContentEncoder>,
    semaphores: &Semaphores,
    logger: &Logger,
) -> Result<Option<(String, ProjectAssetEncoding)>, CreateEncodingError> {
    match encoder {
        None => {
            let identity_asset_encoding = make_project_asset_encoding(
                chunk_upload_target,
                asset_descriptor,
                canister_assets,
                content,
                CONTENT_ENCODING_IDENTITY,
                semaphores,
                logger,
            )
            .await
            .map_err(CreateEncodingError::CreateChunkFailed)?;
            Ok(Some((
                CONTENT_ENCODING_IDENTITY.to_string(),
                identity_asset_encoding,
            )))
        }
        Some(encoder) => {
            let encoded = content.encode(encoder).map_err(|e| {
                EncodeContentFailed(asset_descriptor.key.clone(), encoder.clone(), e)
            })?;
            if encoded.data.len() < content.data.len() {
                let content_encoding = format!("{}", encoder);
                let project_asset_encoding = make_project_asset_encoding(
                    chunk_upload_target,
                    asset_descriptor,
                    canister_assets,
                    &encoded,
                    &content_encoding,
                    semaphores,
                    logger,
                )
                .await
                .map_err(CreateEncodingError::CreateChunkFailed)?;
                Ok(Some((content_encoding, project_asset_encoding)))
            } else {
                Ok(None)
            }
        }
    }
}

async fn make_encodings(
    chunk_upload_target: Option<&ChunkUploader<'_>>,
    asset_descriptor: &AssetDescriptor,
    canister_assets: &HashMap<String, AssetDetails>,
    content: &Content,
    semaphores: &Semaphores,
    logger: &Logger,
) -> Result<HashMap<String, ProjectAssetEncoding>, CreateEncodingError> {
    let mut encoders = vec![None];
    let additional_encoders = asset_descriptor
        .config
        .encodings
        .clone()
        .unwrap_or_else(|| default_encoders(&content.media_type));
    for encoder in additional_encoders {
        encoders.push(Some(encoder));
    }

    let encoding_futures: Vec<_> = encoders
        .iter()
        .map(|maybe_encoder| {
            make_encoding(
                chunk_upload_target,
                asset_descriptor,
                canister_assets,
                content,
                maybe_encoder,
                semaphores,
                logger,
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

async fn make_project_asset(
    chunk_upload_target: Option<&ChunkUploader<'_>>,
    asset_descriptor: AssetDescriptor,
    canister_assets: &HashMap<String, AssetDetails>,
    semaphores: &Semaphores,
    logger: &Logger,
) -> Result<ProjectAsset, CreateProjectAssetError> {
    let file_size = dfx_core::fs::metadata(&asset_descriptor.source)
        .map_err(CreateProjectAssetError::DetermineAssetSizeFailed)?
        .len();
    let permits = std::cmp::max(
        1,
        std::cmp::min(
            ((file_size + 999999) / 1000000) as usize,
            MAX_COST_SINGLE_FILE_MB,
        ),
    );
    let _releaser = semaphores.file.acquire(permits).await;
    let content = Content::load(&asset_descriptor.source)
        .map_err(CreateProjectAssetError::LoadContentFailed)?;

    let encodings = make_encodings(
        chunk_upload_target,
        &asset_descriptor,
        canister_assets,
        &content,
        semaphores,
        logger,
    )
    .await?;

    Ok(ProjectAsset {
        asset_descriptor,
        media_type: content.media_type,
        encodings,
    })
}

pub(crate) async fn make_project_assets(
    chunk_upload_target: Option<&ChunkUploader<'_>>,
    asset_descriptors: Vec<AssetDescriptor>,
    canister_assets: &HashMap<String, AssetDetails>,
    logger: &Logger,
) -> Result<HashMap<String, ProjectAsset>, CreateProjectAssetError> {
    let semaphores = Semaphores::new();

    let project_asset_futures: Vec<_> = asset_descriptors
        .iter()
        .map(|loc| {
            make_project_asset(
                chunk_upload_target,
                loc.clone(),
                canister_assets,
                &semaphores,
                logger,
            )
        })
        .collect();
    let project_assets = try_join_all(project_asset_futures).await?;

    let mut hm = HashMap::new();
    for project_asset in project_assets {
        hm.insert(project_asset.asset_descriptor.key.clone(), project_asset);
    }
    Ok(hm)
}

async fn upload_content_chunks(
    chunk_uploader: &ChunkUploader<'_>,
    asset_descriptor: &AssetDescriptor,
    content: &Content,
    sha256: &Vec<u8>,
    content_encoding: &str,
    semaphores: &Semaphores,
    logger: &Logger,
) -> Result<Vec<Nat>, CreateChunkError> {
    if content.data.is_empty() {
        let empty = vec![];
        let chunk_id = chunk_uploader.create_chunk(&empty, semaphores).await?;
        info!(
            logger,
            "  {}{} 1/1 (0 bytes) sha {}",
            &asset_descriptor.key,
            content_encoding_descriptive_suffix(content_encoding),
            hex::encode(sha256)
        );
        return Ok(vec![chunk_id]);
    }

    let count = (content.data.len() + MAX_CHUNK_SIZE - 1) / MAX_CHUNK_SIZE;
    let chunks_futures: Vec<_> = content
        .data
        .chunks(MAX_CHUNK_SIZE)
        .enumerate()
        .map(|(i, data_chunk)| {
            chunk_uploader
                .create_chunk(data_chunk, semaphores)
                .map_ok(move |chunk_id| {
                    info!(
                        logger,
                        "  {}{} {}/{} ({} bytes) sha {} {}",
                        &asset_descriptor.key,
                        content_encoding_descriptive_suffix(content_encoding),
                        i + 1,
                        count,
                        data_chunk.len(),
                        hex::encode(sha256),
                        &asset_descriptor.config
                    );
                    debug!(logger, "{:?}", &asset_descriptor.config);

                    chunk_id
                })
        })
        .collect();
    try_join_all(chunks_futures).await
}

fn content_encoding_descriptive_suffix(content_encoding: &str) -> String {
    if content_encoding == CONTENT_ENCODING_IDENTITY {
        "".to_string()
    } else {
        format!(" ({})", content_encoding)
    }
}

fn default_encoders(media_type: &Mime) -> Vec<ContentEncoder> {
    match (media_type.type_(), media_type.subtype()) {
        (mime::TEXT, _) | (_, mime::JAVASCRIPT) | (_, mime::HTML) => vec![ContentEncoder::Gzip],
        _ => vec![],
    }
}
