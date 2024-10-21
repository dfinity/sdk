use crate::asset::config::AssetConfig;
use crate::asset::content::Content;
use crate::asset::content_encoder::ContentEncoder;
use crate::batch_upload::semaphores::Semaphores;
use crate::canister_api::methods::chunk::create_chunk;
use crate::canister_api::methods::chunk::create_chunks;
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
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

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
    pub(crate) uploader_chunk_ids: Vec<usize>,
    pub(crate) sha256: Vec<u8>,
    pub(crate) already_in_place: bool,
}

pub(crate) struct ProjectAsset {
    pub(crate) asset_descriptor: AssetDescriptor,
    pub(crate) media_type: Mime,
    pub(crate) encodings: HashMap<String, ProjectAssetEncoding>,
}

enum UploaderState {
    Uploading,
    /// Uploader has uploaded chunks - commit will reference chunk ids to specify asset content
    FinalizedWithUploads,
    /// Uploader has not uploaded chunks - commit will contain asset content directly
    FinalizedWithoutUploads,
}

pub(crate) enum UploaderIdMapping {
    Error(String),
    /// Chunks are uploaded to the canister with these ids
    CanisterChunkIds(Vec<Nat>),
    /// Chunks are not uploaded and should be included in the SetAssetContent operations directly
    IncludeChunksDirectly(Vec<Vec<u8>>),
}

type IdMapping = BTreeMap<usize, Nat>;
type UploadQueue = Vec<(usize, Vec<u8>)>;
pub(crate) struct ChunkUploader<'agent> {
    canister: Canister<'agent>,
    batch_id: Nat,
    api_version: u16,
    chunks: Arc<AtomicUsize>,
    bytes: Arc<AtomicUsize>,
    // maps uploader_chunk_id to canister_chunk_id
    id_mapping: Arc<Mutex<IdMapping>>,
    upload_queue: Arc<Mutex<UploadQueue>>,
    uploader_state: Arc<Mutex<UploaderState>>,
}

impl<'agent> ChunkUploader<'agent> {
    pub(crate) fn new(canister: Canister<'agent>, api_version: u16, batch_id: Nat) -> Self {
        Self {
            canister,
            batch_id,
            api_version,
            chunks: Arc::new(AtomicUsize::new(0)),
            bytes: Arc::new(AtomicUsize::new(0)),
            id_mapping: Arc::new(Mutex::new(BTreeMap::new())),
            upload_queue: Arc::new(Mutex::new(vec![])),
            uploader_state: Arc::new(Mutex::new(UploaderState::Uploading)),
        }
    }

    /// Returns an uploader_chunk_id, which is different from the chunk id on the asset canister.
    /// uploader_chunk_id can be mapped to canister_chunk_id using `uploader_ids_to_canister_chunk_ids`
    /// once `finalize_upload` has completed.
    pub(crate) async fn create_chunk(
        &self,
        contents: &[u8],
        semaphores: &Semaphores,
    ) -> Result<usize, CreateChunkError> {
        let uploader_chunk_id = self.chunks.fetch_add(1, Ordering::SeqCst);
        self.bytes.fetch_add(contents.len(), Ordering::SeqCst);
        if contents.len() == MAX_CHUNK_SIZE || self.api_version < 2 {
            let canister_chunk_id =
                create_chunk(&self.canister, &self.batch_id, contents, semaphores).await?;
            let mut map = self.id_mapping.lock().await;
            map.insert(uploader_chunk_id, canister_chunk_id);
            Ok(uploader_chunk_id)
        } else {
            self.add_to_upload_queue(uploader_chunk_id, contents).await;
            // Larger `max_retained_bytes` leads to batches that are filled closer to the max size.
            // `4 * MAX_CHUNK_SIZE` leads to a pretty small memory footprint but still offers solid fill rates.
            // Mini experiment:
            //  - Tested with: `for i in $(seq 1 50); do dd if=/dev/urandom of="src/hello_frontend/assets/file_$i.bin" bs=$(shuf -i 1-2000000 -n 1) count=1; done && dfx deploy hello_frontend`
            //  - Result: Roughly 15% of batches under 90% full.
            // With other byte ranges (e.g. `shuf -i 1-3000000 -n 1`) stats improve significantly
            self.upload_chunks(4 * MAX_CHUNK_SIZE, usize::MAX, semaphores)
                .await?;
            Ok(uploader_chunk_id)
        }
    }

    pub(crate) async fn finalize_upload(
        &self,
        semaphores: &Semaphores,
    ) -> Result<(), CreateChunkError> {
        let queue = self.upload_queue.lock().await;
        let mut uploader_state = self.uploader_state.lock().await;

        // Can skip upload if every chunk submitted for uploading is still in the queue.
        // Additionally, chunks in the queue are small enough that there is plenty of space in the commit message to include all of them.
        let skip_upload = queue.len() == self.chunks.fetch_add(0, Ordering::SeqCst)
            && queue.iter().map(|(_, chunk)| chunk.len()).sum::<usize>() < MAX_CHUNK_SIZE / 2;
        drop(queue);
        // Potential for further improvement: unconditional upload_chunks(MAX_CHUNK_SIZE / 2, usize::MAX, semaphores)
        // Then allow mix of uploaded chunks and asset content that is part of the commit args.

        if skip_upload {
            *uploader_state = UploaderState::FinalizedWithoutUploads;
        } else {
            self.upload_chunks(0, 0, semaphores).await?;
            *uploader_state = UploaderState::FinalizedWithUploads;
        }
        Ok(())
    }

    pub(crate) fn bytes(&self) -> usize {
        self.bytes.load(Ordering::SeqCst)
    }
    pub(crate) fn chunks(&self) -> usize {
        self.chunks.load(Ordering::SeqCst)
    }

    /// Call only after `finalize_upload` has completed
    pub(crate) async fn uploader_ids_to_canister_chunk_ids(
        &self,
        uploader_ids: &[usize],
    ) -> UploaderIdMapping {
        let uploader_state = self.uploader_state.lock().await;
        match *uploader_state {
            UploaderState::Uploading => UploaderIdMapping::Error(
                "Bug: Tried to map uploader ids to canister ids before finalizing".to_string(),
            ),
            UploaderState::FinalizedWithUploads => {
                let mapping = self.id_mapping.lock().await;
                let ids = uploader_ids
                    .iter()
                    .map(|id| {
                        mapping
                            .get(id)
                            .expect("Chunk uploader did not upload all chunks but is not aware of it. This is a bug.")
                            .clone()
                    })
                    .collect();
                UploaderIdMapping::CanisterChunkIds(ids)
            }
            UploaderState::FinalizedWithoutUploads => {
                let queue = self.upload_queue.lock().await;
                match uploader_ids
                    .iter()
                    .map(|uploader_id| {
                        queue.iter().find_map(|(id, content)| {
                            if id == uploader_id {
                                Some(content.clone())
                            } else {
                                None
                            }
                        }).ok_or_else(|| format!("Chunk uploader does not have a chunk with uploader id {uploader_id}. This is a bug."))
                    })
                    .collect() {
                        Ok(asset_content) =>  UploaderIdMapping::IncludeChunksDirectly(asset_content),
                        Err(err) => UploaderIdMapping::Error(err)
                    }
            }
        }
    }

    async fn add_to_upload_queue(&self, uploader_chunk_id: usize, contents: &[u8]) {
        let mut queue = self.upload_queue.lock().await;
        queue.push((uploader_chunk_id, contents.into()));
    }

    /// Calls `upload_chunks` with batches of chunks from `self.upload_queue` until at most `max_retained_bytes`
    /// bytes and at most `max_retained_chunks` chunks remain in the upload queue. Larger values
    /// will lead to better batch fill rates but also leave a larger memory footprint.
    async fn upload_chunks(
        &self,
        max_retained_bytes: usize,
        max_retained_chunks: usize,
        semaphores: &Semaphores,
    ) -> Result<(), CreateChunkError> {
        let mut queue = self.upload_queue.lock().await;

        let mut batches = vec![];
        while queue
            .iter()
            .map(|(_, content)| content.len())
            .sum::<usize>()
            > max_retained_bytes
            || queue.len() > max_retained_chunks
        {
            // Greedily fills batch with the largest chunk that fits
            queue.sort_unstable_by_key(|(_, content)| content.len());
            let mut batch = vec![];
            let mut batch_size = 0;
            for (uploader_chunk_id, content) in std::mem::take(&mut *queue).into_iter().rev() {
                if content.len() <= MAX_CHUNK_SIZE - batch_size {
                    batch_size += content.len();
                    batch.push((uploader_chunk_id, content));
                } else {
                    queue.push((uploader_chunk_id, content));
                }
            }
            batches.push(batch);
        }

        try_join_all(batches.into_iter().map(|chunks| async move {
            let (uploader_chunk_ids, chunks): (Vec<_>, Vec<_>) = chunks.into_iter().unzip();
            let canister_chunk_ids =
                create_chunks(&self.canister, &self.batch_id, chunks, semaphores).await?;
            let mut map = self.id_mapping.lock().await;
            for (uploader_id, canister_id) in uploader_chunk_ids
                .into_iter()
                .zip(canister_chunk_ids.into_iter())
            {
                map.insert(uploader_id, canister_id);
            }
            Ok(())
        }))
        .await?;

        Ok(())
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

    let uploader_chunk_ids = if already_in_place {
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
        uploader_chunk_ids,
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
    encoder: &ContentEncoder,
    force_encoding: bool,
    semaphores: &Semaphores,
    logger: &Logger,
) -> Result<Option<(String, ProjectAssetEncoding)>, CreateEncodingError> {
    match encoder {
        ContentEncoder::Identity => {
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
        encoder => {
            let encoded = content.encode(encoder).map_err(|e| {
                EncodeContentFailed(asset_descriptor.key.clone(), encoder.to_owned(), e)
            })?;
            if force_encoding || encoded.data.len() < content.data.len() {
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
    let encoders = asset_descriptor
        .config
        .encodings
        .clone()
        .unwrap_or_else(|| default_encoders(&content.media_type));
    // The identity encoding is always uploaded if it's in the list of chosen encodings.
    // Other encoding are only uploaded if they save bytes compared to identity.
    // The encoding is forced through the filter if there is no identity encoding to compare against.
    let force_encoding = !encoders.contains(&ContentEncoder::Identity);

    let encoding_futures: Vec<_> = encoders
        .iter()
        .map(|encoder| {
            make_encoding(
                chunk_upload_target,
                asset_descriptor,
                canister_assets,
                content,
                encoder,
                force_encoding,
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
    let file_size = dfx_core::fs::metadata(&asset_descriptor.source)?.len();
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
    if let Some(uploader) = chunk_upload_target {
        uploader.finalize_upload(&semaphores).await.map_err(|err| {
            CreateProjectAssetError::CreateEncodingError(CreateEncodingError::CreateChunkFailed(
                err,
            ))
        })?;
    }

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
) -> Result<Vec<usize>, CreateChunkError> {
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
        (mime::TEXT, _) | (_, mime::JAVASCRIPT) | (_, mime::HTML) => {
            vec![ContentEncoder::Identity, ContentEncoder::Gzip]
        }
        _ => vec![ContentEncoder::Identity],
    }
}
