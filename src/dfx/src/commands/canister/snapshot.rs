use std::{
    fmt::{self, Display, Formatter},
    path::PathBuf,
    str::FromStr,
};

use anyhow::{Context, Error, anyhow, bail};
use backoff::ExponentialBackoff;
use backoff::future::retry;
use candid::Principal;
use clap::{Parser, Subcommand};
use dfx_core::{
    identity::CallSender,
    json::{load_json_file, save_json_file},
};
use ic_management_canister_types::{
    CanisterStatusType, LoadCanisterSnapshotArgs, ReadCanisterSnapshotDataArgs,
    ReadCanisterSnapshotMetadataArgs, ReadCanisterSnapshotMetadataResult, SnapshotDataKind,
    SnapshotDataOffset, UploadCanisterSnapshotDataArgs, UploadCanisterSnapshotMetadataArgs,
    UploadCanisterSnapshotMetadataResult,
};
use indicatif::{HumanBytes, ProgressStyle};
use itertools::Itertools;
use slog::{debug, error, info};
use time::{OffsetDateTime, macros::format_description};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::lib::{
    environment::Environment,
    error::{DfxError, DfxResult},
    operations::canister::{
        delete_canister_snapshot, get_canister_status, list_canister_snapshots,
        load_canister_snapshot, read_canister_snapshot_data, read_canister_snapshot_metadata,
        take_canister_snapshot, upload_canister_snapshot_data, upload_canister_snapshot_metadata,
    },
    retryable::retryable,
    root_key::fetch_root_key_if_needed,
};
use crate::util::clap::parsers::directory_parser;

#[derive(Parser)]
pub struct SnapshotOpts {
    #[command(subcommand)]
    subcmd: SnapshotSubcommand,
}

/// Controls canister snapshots that can reset a canister to an earlier state of execution.
#[derive(Subcommand)]
enum SnapshotSubcommand {
    /// Creates a new snapshot of a canister. The canister must be stopped.
    Create {
        /// The canister to snapshot.
        canister: String,
        /// If a snapshot ID is specified, this snapshot will replace it and reuse the ID.
        #[arg(long)]
        replace: Option<SnapshotId>,
    },
    /// Loads a canister snapshot, overwriting its execution state. All data since that snapshot will be lost. The canister must be stopped.
    Load {
        /// The canister to load the snapshot in.
        canister: String,
        /// The ID of the snapshot to load.
        snapshot: SnapshotId,
    },
    /// Lists a canister's existing snapshots.
    List {
        /// The canister to list snapshots from.
        canister: String,
    },
    /// Deletes a snapshot from a canister.
    Delete {
        /// The canister to delete the snapshot from.
        canister: String,
        /// The ID of the snapshot to delete.
        snapshot: SnapshotId,
    },
    /// Downloads an existing snapshot from a canister into a given directory.
    Download {
        /// The canister to download the snapshot from.
        canister: String,
        /// The ID of the snapshot to download.
        snapshot: SnapshotId,
        /// The directory to download the snapshot to.
        #[arg(long, value_parser = directory_parser)]
        dir: PathBuf,
    },
    /// Uploads a downloaded snapshot from a given directory to a canister.
    Upload {
        /// The canister to upload the snapshot to.
        canister: String,
        /// If a snapshot ID is specified, this snapshot will replace it and reuse the ID.
        #[arg(long)]
        replace: Option<SnapshotId>,
        /// The directory to upload the snapshot from.
        #[arg(long, value_parser = directory_parser)]
        dir: PathBuf,
    },
}

#[derive(Clone)]
struct SnapshotId(Vec<u8>);

impl Display for SnapshotId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&hex::encode(&self.0))
    }
}

impl FromStr for SnapshotId {
    type Err = DfxError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(hex::decode(s)?))
    }
}

impl From<UploadCanisterSnapshotMetadataResult> for SnapshotId {
    fn from(canister_snapshot_id: UploadCanisterSnapshotMetadataResult) -> Self {
        SnapshotId(canister_snapshot_id.snapshot_id)
    }
}

pub async fn exec(
    env: &dyn Environment,
    opts: SnapshotOpts,
    call_sender: &CallSender,
) -> DfxResult {
    fetch_root_key_if_needed(env).await?;
    match opts.subcmd {
        SnapshotSubcommand::Create { canister, replace } => {
            create(env, canister, replace, call_sender).await?
        }
        SnapshotSubcommand::Load { canister, snapshot } => {
            load(env, canister, snapshot, call_sender).await?
        }
        SnapshotSubcommand::Delete { canister, snapshot } => {
            delete(env, canister, snapshot, call_sender).await?
        }
        SnapshotSubcommand::List { canister } => list(env, canister, call_sender).await?,
        SnapshotSubcommand::Download {
            canister,
            snapshot,
            dir,
        } => download(env, canister, snapshot, dir, call_sender).await?,
        SnapshotSubcommand::Upload {
            canister,
            replace,
            dir,
        } => upload(env, canister, replace, dir, call_sender).await?,
    }
    Ok(())
}

fn ensure_status(status: CanisterStatusType, canister: &str, phrasing: &str) -> DfxResult {
    match status {
        CanisterStatusType::Stopped => {}
        CanisterStatusType::Running => bail!(
            "Canister {canister} is running and snapshots should not be {phrasing} running canisters. Run `dfx canister stop` first"
        ),
        CanisterStatusType::Stopping => bail!(
            "Canister {canister} is stopping but is not yet stopped. Wait a few seconds and try again"
        ),
    }
    Ok(())
}

async fn create(
    env: &dyn Environment,
    canister: String,
    replace: Option<SnapshotId>,
    call_sender: &CallSender,
) -> DfxResult {
    let canister_id = canister
        .parse()
        .or_else(|_| env.get_canister_id_store()?.get(&canister))?;
    let status = get_canister_status(env, canister_id, call_sender)
        .await
        .with_context(|| format!("Could not retrieve status of canister {canister}"))?;
    ensure_status(status.status, &canister, "taken of")?;
    let id = take_canister_snapshot(
        env,
        canister_id,
        replace.as_ref().map(|x| &*x.0),
        call_sender,
    )
    .await
    .with_context(|| format!("Failed to take snapshot of canister {canister}"))?;
    info!(
        env.get_logger(),
        "Created a new snapshot of canister {canister}. Snapshot ID: {}",
        SnapshotId(id.id)
    );
    Ok(())
}

async fn load(
    env: &dyn Environment,
    canister: String,
    snapshot: SnapshotId,
    call_sender: &CallSender,
) -> DfxResult {
    let canister_id = canister
        .parse()
        .or_else(|_| env.get_canister_id_store()?.get(&canister))?;
    let status = get_canister_status(env, canister_id, call_sender)
        .await
        .with_context(|| format!("Could not retrieve status of canister {canister}"))?;
    ensure_status(status.status, &canister, "applied to")?;
    let load_args = LoadCanisterSnapshotArgs {
        canister_id,
        snapshot_id: snapshot.0.clone(),
        sender_canister_version: None,
    };
    load_canister_snapshot(env, canister_id, &load_args, call_sender)
        .await
        .with_context(|| format!("Failed to load snapshot {snapshot} in canister {canister}"))?;
    info!(
        env.get_logger(),
        "Loaded snapshot {snapshot} in canister {canister}"
    );
    Ok(())
}

async fn delete(
    env: &dyn Environment,
    canister: String,
    snapshot: SnapshotId,
    call_sender: &CallSender,
) -> DfxResult {
    let canister_id = canister
        .parse()
        .or_else(|_| env.get_canister_id_store()?.get(&canister))?;
    delete_canister_snapshot(env, canister_id, &snapshot.0, call_sender)
        .await
        .with_context(|| format!("Failed to delete snapshot {snapshot} in canister {canister}"))?;
    info!(
        env.get_logger(),
        "Deleted snapshot {snapshot} from canister {canister}"
    );
    Ok(())
}

async fn list(env: &dyn Environment, canister: String, call_sender: &CallSender) -> DfxResult {
    let canister_id = canister
        .parse()
        .or_else(|_| env.get_canister_id_store()?.get(&canister))?;
    let snapshots = list_canister_snapshots(env, canister_id, call_sender)
        .await
        .with_context(|| format!("Failed to retrieve snapshot list from canister {canister}"))?;
    if snapshots.is_empty() {
        info!(
            env.get_logger(),
            "No snapshots found in canister {canister}"
        );
    } else {
        let time_fmt = format_description!("[year]-[month]-[day] [hour]:[minute]:[second] UTC");
        let snapshots = snapshots.into_iter().format_with("\n", |s, f| {
            f(&format_args!(
                "{}: {}, taken at {}",
                SnapshotId(s.id),
                HumanBytes(s.total_size),
                OffsetDateTime::from_unix_timestamp_nanos(s.taken_at_timestamp as i128)
                    .unwrap()
                    .format(&time_fmt)
                    .unwrap()
            ))
        });
        info!(env.get_logger(), "{snapshots}");
    }
    Ok(())
}

async fn download(
    env: &dyn Environment,
    canister: String,
    snapshot: SnapshotId,
    dir: PathBuf,
    call_sender: &CallSender,
) -> DfxResult {
    check_dir(&dir)?;

    let canister_id = canister
        .parse()
        .or_else(|_| env.get_canister_id_store()?.get(&canister))?;

    // Store metadata.
    let metadata_args = ReadCanisterSnapshotMetadataArgs {
        canister_id,
        snapshot_id: snapshot.0.clone(),
    };
    let metadata = read_canister_snapshot_metadata(env, canister_id, &metadata_args, call_sender)
        .await
        .with_context(|| {
            format!("Failed to read metadata from snapshot {snapshot} in canister {canister}")
        })?;
    let metadata_file = dir.join("metadata.json");
    save_json_file(&metadata_file, &metadata)?;
    debug!(
        env.get_logger(),
        "Snapshot metadata saved to '{}'",
        metadata_file.display()
    );

    let retry_policy = ExponentialBackoff::default();

    // Store Wasm module.
    store_data(
        env,
        &canister,
        canister_id,
        &snapshot,
        BlobKind::WasmModule,
        metadata.wasm_module_size as usize,
        dir.join("wasm_module.bin"),
        retry_policy.clone(),
        call_sender,
    )
    .await?;

    // Store Wasm memory.
    store_data(
        env,
        &canister,
        canister_id,
        &snapshot,
        BlobKind::MainMemory,
        metadata.wasm_memory_size as usize,
        dir.join("wasm_memory.bin"),
        retry_policy.clone(),
        call_sender,
    )
    .await?;

    // Store stable memory.
    if metadata.stable_memory_size > 0 {
        store_data(
            env,
            &canister,
            canister_id,
            &snapshot,
            BlobKind::StableMemory,
            metadata.stable_memory_size as usize,
            dir.join("stable_memory.bin"),
            retry_policy.clone(),
            call_sender,
        )
        .await?;
    }

    // Store Wasm chunks.
    if !metadata.wasm_chunk_store.is_empty() {
        let wasm_chunk_store_dir = dir.join("wasm_chunk_store");
        std::fs::create_dir(&wasm_chunk_store_dir).with_context(|| {
            format!(
                "Failed to create directory '{}'",
                wasm_chunk_store_dir.display()
            )
        })?;

        for chunk_hash in metadata.wasm_chunk_store {
            let hash_str = hex::encode(&chunk_hash.hash);
            let chunk_file = wasm_chunk_store_dir.join(format!("{hash_str}.bin"));

            let chunk = retry(retry_policy.clone(), || async {
                let data_args = ReadCanisterSnapshotDataArgs {
                    canister_id,
                    snapshot_id: snapshot.0.clone(),
                    kind: SnapshotDataKind::WasmChunk {
                        hash: chunk_hash.hash.clone(),
                    },
                };
                match read_canister_snapshot_data(
                    env,
                    canister_id,
                    &data_args,
                    call_sender,
                )
                .await {
                    Ok(chunk) => Ok(chunk),
                    Err(_error) => Err(backoff::Error::transient(anyhow!(
                        "Failed to read data from snapshot {snapshot} from canister {canister} for chunk {hash_str}"
                    ))),
                }
            })
            .await?
            .chunk;

            std::fs::write(&chunk_file, &chunk).with_context(|| {
                format!("Failed to write chunk data to '{}'", chunk_file.display())
            })?;
            debug!(
                env.get_logger(),
                "Wasm chunk data saved to '{}'",
                chunk_file.display()
            );
        }
    }

    info!(
        env.get_logger(),
        "Snapshot {snapshot} in canister {} saved to '{}'",
        canister,
        dir.display()
    );

    Ok(())
}

async fn upload(
    env: &dyn Environment,
    canister: String,
    replace: Option<SnapshotId>,
    dir: PathBuf,
    call_sender: &CallSender,
) -> DfxResult {
    let canister_id = canister
        .parse()
        .or_else(|_| env.get_canister_id_store()?.get(&canister))?;

    // Upload snapshot metadata.
    let metadata: ReadCanisterSnapshotMetadataResult = load_json_file(&dir.join("metadata.json"))?;
    let metadata_args = UploadCanisterSnapshotMetadataArgs {
        canister_id,
        replace_snapshot: replace.as_ref().map(|x| x.0.clone()),
        wasm_module_size: metadata.wasm_module_size,
        globals: metadata.globals,
        wasm_memory_size: metadata.wasm_memory_size,
        stable_memory_size: metadata.stable_memory_size,
        certified_data: metadata.certified_data,
        global_timer: metadata.global_timer,
        on_low_wasm_memory_hook_status: metadata.on_low_wasm_memory_hook_status,
    };
    let snapshot_id =
        upload_canister_snapshot_metadata(env, canister_id, &metadata_args, call_sender)
            .await
            .with_context(|| format!("Failed to upload snapshot metadata to canister {canister}"))?
            .into();
    debug!(
        env.get_logger(),
        "Snapshot metadata uploaded to canister {canister} with Snapshot ID: {snapshot_id}"
    );

    let retry_policy = ExponentialBackoff::default();

    // Upload Wasm module.
    upload_data(
        env,
        &canister,
        canister_id,
        &snapshot_id,
        BlobKind::WasmModule,
        dir.join("wasm_module.bin"),
        retry_policy.clone(),
        call_sender,
    )
    .await?;

    // Upload Wasm memory.
    upload_data(
        env,
        &canister,
        canister_id,
        &snapshot_id,
        BlobKind::MainMemory,
        dir.join("wasm_memory.bin"),
        retry_policy.clone(),
        call_sender,
    )
    .await?;

    // Upload stable memory.
    if metadata.stable_memory_size > 0 {
        upload_data(
            env,
            &canister,
            canister_id,
            &snapshot_id,
            BlobKind::StableMemory,
            dir.join("stable_memory.bin"),
            retry_policy.clone(),
            call_sender,
        )
        .await?;
    }

    // Upload Wasm chunks.
    if !metadata.wasm_chunk_store.is_empty() {
        let wasm_chunk_store_dir = dir.join("wasm_chunk_store");
        for chunk_hash in metadata.wasm_chunk_store {
            let hash_str = hex::encode(&chunk_hash.hash);
            let chunk_file = wasm_chunk_store_dir.join(format!("{hash_str}.bin"));
            let chunk_data = std::fs::read(&chunk_file).with_context(|| {
                format!("Failed to read Wasm chunk from '{}'", chunk_file.display())
            })?;

            retry(retry_policy.clone(), || async {
                let data_args = UploadCanisterSnapshotDataArgs {
                    canister_id,
                    snapshot_id: snapshot_id.0.clone(),
                    kind: SnapshotDataOffset::WasmChunk,
                    chunk: chunk_data.clone(),
                };
                match upload_canister_snapshot_data(
                    env,
                    canister_id,
                    &data_args,
                    call_sender,
                )
                .await
                {
                    Ok(_) => Ok(()),
                    Err(_error) => Err(backoff::Error::transient(anyhow!(
                        "Failed to upload Wasm chunk {hash_str} to snapshot {snapshot_id} in canister {canister}"
                    ))),
                }
            })
            .await?;
            debug!(
                env.get_logger(),
                "Snapshot Wasm chunk {} uploaded to canister {} with Snapshot ID: {}",
                hex::encode(&chunk_hash.hash),
                canister,
                snapshot_id
            );
        }
    }

    info!(
        env.get_logger(),
        "Uploaded a snapshot of canister {canister}. Snapshot ID: {}", snapshot_id
    );

    Ok(())
}

fn check_dir(dir: &PathBuf) -> DfxResult {
    // Check if the directory is empty.
    let mut entries = std::fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory '{}'", dir.display()))?;
    if entries.next().is_some() {
        bail!("Directory '{}' is not empty", dir.display());
    }

    // Check if the directory is writable.
    let temp_file = dir.join(".snapshot_write_test");
    match std::fs::File::create(&temp_file) {
        Ok(_) => {
            std::fs::remove_file(&temp_file).with_context(|| {
                format!(
                    "Failed to remove temporary test file '{}'",
                    temp_file.display()
                )
            })?;
        }
        Err(e) => {
            bail!("Directory '{}' is not writable: {}", dir.display(), e);
        }
    }

    Ok(())
}

#[derive(Debug)]
enum BlobKind {
    WasmModule,
    MainMemory,
    StableMemory,
}

const MAX_CHUNK_SIZE: usize = 2_000_000;

async fn store_data(
    env: &dyn Environment,
    canister: &str,
    canister_id: Principal,
    snapshot_id: &SnapshotId,
    blob_kind: BlobKind,
    length: usize,
    file_path: PathBuf,
    retry_policy: ExponentialBackoff,
    call_sender: &CallSender,
) -> DfxResult {
    let message = match blob_kind {
        BlobKind::WasmModule => "Wasm module",
        BlobKind::MainMemory => "Wasm memory",
        BlobKind::StableMemory => "stable memory",
    };

    info!(env.get_logger(), "Downloading {message}");

    write_blob(
        env,
        canister,
        canister_id,
        snapshot_id,
        blob_kind,
        length,
        &file_path,
        retry_policy.clone(),
        call_sender,
    )
    .await
    .with_context(|| {
        format!("Failed to download {message} from snapshot {snapshot_id} in canister {canister}")
    })?;

    info!(
        env.get_logger(),
        "\nThe {message} has been saved to '{}'",
        file_path.display()
    );

    Ok(())
}

async fn write_blob(
    env: &dyn Environment,
    canister: &str,
    canister_id: Principal,
    snapshot: &SnapshotId,
    blob_kind: BlobKind,
    length: usize,
    file_path: &PathBuf,
    retry_policy: ExponentialBackoff,
    call_sender: &CallSender,
) -> DfxResult {
    let pb = get_progress_bar();
    pb.set_length(length as u64);

    let mut file = tokio::fs::File::create(file_path).await?;
    let mut offset = 0;
    while offset < length {
        let chunk_size = std::cmp::min(length - offset, MAX_CHUNK_SIZE);
        let kind = match blob_kind {
            BlobKind::WasmModule => SnapshotDataKind::WasmModule {
                offset: offset as u64,
                size: chunk_size as u64,
            },
            BlobKind::MainMemory => SnapshotDataKind::WasmMemory {
                offset: offset as u64,
                size: chunk_size as u64,
            },
            BlobKind::StableMemory => SnapshotDataKind::StableMemory {
                offset: offset as u64,
                size: chunk_size as u64,
            },
        };

        let data_args = ReadCanisterSnapshotDataArgs {
            canister_id,
            snapshot_id: snapshot.0.clone(),
            kind,
        };
        let chunk = retry(retry_policy.clone(), || async {
            match read_canister_snapshot_data(env, canister_id, &data_args, call_sender).await {
                Ok(chunk) => Ok(chunk),
                Err(error) if is_retryable(&error) => {
                    error!(
                        env.get_logger(),
                        "Failed to read {:?} from snapshot {snapshot} in canister {canister}.",
                        blob_kind,
                    );
                    Err(backoff::Error::transient(anyhow!(error)))
                }
                Err(error) => Err(backoff::Error::permanent(anyhow!(error))),
            }
        })
        .await?
        .chunk;
        file.write_all(&chunk).await?;

        offset += chunk_size;
        pb.set_position(offset as u64);
    }
    file.flush().await?;

    pb.finish();

    Ok(())
}

async fn upload_data(
    env: &dyn Environment,
    canister: &str,
    canister_id: Principal,
    snapshot_id: &SnapshotId,
    blob_kind: BlobKind,
    file_path: PathBuf,
    retry_policy: ExponentialBackoff,
    call_sender: &CallSender,
) -> DfxResult {
    let message = match blob_kind {
        BlobKind::WasmModule => "Wasm module",
        BlobKind::MainMemory => "Wasm memory",
        BlobKind::StableMemory => "stable memory",
    };

    info!(env.get_logger(), "Uploading {message}");

    upload_blob(
        env,
        canister,
        canister_id,
        snapshot_id,
        blob_kind,
        &file_path,
        retry_policy.clone(),
        call_sender,
    )
    .await
    .with_context(|| {
        format!("Failed to upload {message} to snapshot {snapshot_id} in canister {canister}")
    })?;
    info!(
        env.get_logger(),
        "The {message} has been uploaded from '{}'",
        file_path.display()
    );

    Ok(())
}

async fn upload_blob(
    env: &dyn Environment,
    canister: &str,
    canister_id: Principal,
    snapshot: &SnapshotId,
    blob_kind: BlobKind,
    file_path: &PathBuf,
    retry_policy: ExponentialBackoff,
    call_sender: &CallSender,
) -> DfxResult {
    let length = std::fs::metadata(file_path)
        .with_context(|| format!("Failed to get length of file '{}'", file_path.display()))?
        .len() as usize;

    let pb = get_progress_bar();
    pb.set_length(length as u64);

    let mut file = tokio::fs::File::open(file_path)
        .await
        .with_context(|| format!("Failed to open file '{}' for reading", file_path.display()))?;
    let mut offset = 0;
    while offset < length {
        let chunk_size = std::cmp::min(length - offset, MAX_CHUNK_SIZE);
        let kind = match blob_kind {
            BlobKind::WasmModule => SnapshotDataOffset::WasmModule {
                offset: offset as u64,
            },
            BlobKind::MainMemory => SnapshotDataOffset::WasmMemory {
                offset: offset as u64,
            },
            BlobKind::StableMemory => SnapshotDataOffset::StableMemory {
                offset: offset as u64,
            },
        };

        let mut chunk = vec![0u8; chunk_size];
        file.read_exact(&mut chunk).await?;

        let data_args = UploadCanisterSnapshotDataArgs {
            canister_id,
            snapshot_id: snapshot.0.clone(),
            kind,
            chunk,
        };
        retry(retry_policy.clone(), || async {
            match upload_canister_snapshot_data(env, canister_id, &data_args, call_sender).await {
                Ok(_) => Ok(()),
                Err(error) if is_retryable(&error) => {
                    error!(
                        env.get_logger(),
                        "Failed to upload {:?} to snapshot {snapshot} in canister {canister}.",
                        blob_kind,
                    );
                    Err(backoff::Error::transient(anyhow!(error)))
                }
                Err(error) => Err(backoff::Error::permanent(anyhow!(error))),
            }
        })
        .await?;
        offset += chunk_size;
        pb.set_position(offset as u64);
    }

    pb.finish();

    Ok(())
}

fn is_retryable(error: &Error) -> bool {
    if let Some(agent_err) = error.downcast_ref::<ic_agent::AgentError>() {
        return retryable(agent_err);
    }

    false
}

fn get_progress_bar() -> indicatif::ProgressBar {
    let pb = indicatif::ProgressBar::new(0);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        .expect("Failed to set template string")
        .progress_chars("#>-"));
    pb
}
