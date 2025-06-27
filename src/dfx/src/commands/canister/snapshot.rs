use std::{
    fmt::{self, Display, Formatter},
    path::PathBuf,
    str::FromStr,
};

use anyhow::{bail, Context};
use candid::Principal;
use clap::{Parser, Subcommand};
use dfx_core::identity::CallSender;
use ic_utils::interfaces::management_canister::{
    CanisterStatus, SnapshotDataKind, SnapshotMetadata,
};
use indicatif::HumanBytes;
use itertools::Itertools;
use slog::{debug, info};
use time::{macros::format_description, OffsetDateTime};

use crate::lib::{
    environment::Environment,
    error::{DfxError, DfxResult},
    operations::canister::{
        delete_canister_snapshot, get_canister_status, list_canister_snapshots,
        load_canister_snapshot, read_canister_snapshot_data, read_canister_snapshot_metadata,
        take_canister_snapshot, upload_canister_snapshot_metadata,
    },
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
    Download {
        /// The canister to download the snapshot from.
        canister: String,
        /// The ID of the snapshot to download.
        snapshot: SnapshotId,
        /// The directory to download the snapshot to.
        #[arg(long, value_parser = directory_parser)]
        dir: PathBuf,
    },
    Upload {
        /// The canister to upload the snapshot to.
        canister: String,
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
        SnapshotSubcommand::Upload { canister, dir } => {
            upload(env, canister, dir, call_sender).await?
        }
    }
    Ok(())
}

fn ensure_status(status: CanisterStatus, canister: &str, phrasing: &str) -> DfxResult {
    match status {
        CanisterStatus::Stopped => {}
        CanisterStatus::Running => bail!("Canister {canister} is running and snapshots should not be {phrasing} running canisters. Run `dfx canister stop` first"),
        CanisterStatus::Stopping => bail!("Canister {canister} is stopping but is not yet stopped. Wait a few seconds and try again"),
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
    load_canister_snapshot(env, canister_id, &snapshot.0, call_sender)
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
    let metadata = read_canister_snapshot_metadata(env, canister_id, &snapshot.0, call_sender)
        .await
        .with_context(|| {
            format!(
                "Failed to read metadata from snapshot {snapshot} in canister {}",
                canister_id.to_text()
            )
        })?;
    let metadata_file = dir.join("metadata.json");
    let metadata_json = serde_json::to_string_pretty(&metadata)?;
    std::fs::write(&metadata_file, metadata_json).with_context(|| {
        format!(
            "Failed to write snapshot metadata to '{}'",
            metadata_file.display()
        )
    })?;
    debug!(
        env.get_logger(),
        "Snapshot metadata saved to '{}'",
        metadata_file.display()
    );

    // Store Wasm module.
    let wasm_module = read_blob(
        env,
        canister_id,
        &snapshot,
        BlobKind::WasmModule,
        metadata.wasm_module_size as usize,
        call_sender,
    )
    .await
    .with_context(|| {
        format!(
            "Failed to read Wasm module from snapshot {snapshot} in canister {}",
            canister_id.to_text(),
        )
    })?;
    let wasm_module_file = dir.join("wasm_module.bin");
    std::fs::write(&wasm_module_file, &wasm_module).with_context(|| {
        format!(
            "Failed to write Wasm module to '{}'",
            wasm_module_file.display()
        )
    })?;
    debug!(
        env.get_logger(),
        "Wasm module saved to '{}'",
        wasm_module_file.display()
    );

    // Store Wasm memory.
    let wasm_memory = read_blob(
        env,
        canister_id,
        &snapshot,
        BlobKind::MainMemory,
        metadata.wasm_memory_size as usize,
        call_sender,
    )
    .await
    .with_context(|| {
        format!(
            "Failed to read Wasm memory from snapshot {snapshot} in canister {}",
            canister_id.to_text(),
        )
    })?;
    let wasm_memory_file = dir.join("wasm_memory.bin");
    std::fs::write(&wasm_memory_file, &wasm_memory).with_context(|| {
        format!(
            "Failed to write Wasm memory to '{}'",
            wasm_memory_file.display()
        )
    })?;
    debug!(
        env.get_logger(),
        "Wasm memory saved to '{}'",
        wasm_memory_file.display()
    );

    // Store stable memory.
    if metadata.stable_memory_size > 0 {
        let stable_memory = read_blob(
            env,
            canister_id,
            &snapshot,
            BlobKind::StableMemory,
            metadata.stable_memory_size as usize,
            call_sender,
        )
        .await
        .with_context(|| {
            format!(
                "Failed to read stable memory from snapshot {snapshot} in canister {}",
                canister_id.to_text(),
            )
        })?;
        let stable_memory_file = dir.join("stable_memory.bin");
        std::fs::write(&stable_memory_file, &stable_memory).with_context(|| {
            format!(
                "Failed to write stable memory to '{}'",
                stable_memory_file.display()
            )
        })?;
        debug!(
            env.get_logger(),
            "Stable memory saved to '{}'",
            stable_memory_file.display()
        );
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
            let chunk_file =
                wasm_chunk_store_dir.join(format!("{}.bin", hex::encode(&chunk_hash.hash)));

            let chunk = read_canister_snapshot_data(
                env,
                canister_id,
                &snapshot.0,
                &SnapshotDataKind::WasmChunk {
                    hash: chunk_hash.hash,
                },
                call_sender,
            )
            .await
            .with_context(|| {
                format!(
                    "Failed to read data from snapshot {snapshot} from canister {}",
                    canister_id.to_text()
                )
            })?
            .chunk;
            std::fs::write(&chunk_file, &chunk)
                .with_context(|| format!("Failed to write chunk to '{}'", chunk_file.display()))?;
            debug!(
                env.get_logger(),
                "Wasm chunk saved to '{}'",
                chunk_file.display()
            );
        }
    }

    info!(
        env.get_logger(),
        "Snapshot {snapshot} in canister {} saved to '{}'",
        canister_id.to_text(),
        dir.display()
    );

    Ok(())
}

async fn upload(
    env: &dyn Environment,
    canister: String,
    dir: PathBuf,
    call_sender: &CallSender,
) -> DfxResult {
    let canister_id = canister
        .parse()
        .or_else(|_| env.get_canister_id_store()?.get(&canister))?;

    // Upload snapshot metadata.
    let metadata_file = dir.join("metadata.json");
    let metadata: SnapshotMetadata =
        serde_json::from_str(&std::fs::read_to_string(&metadata_file).with_context(|| {
            format!(
                "Failed to read snapshot metadata from '{}'",
                metadata_file.display()
            )
        })?)
        .with_context(|| {
            format!(
                "Failed to deserialize snapshot metadata from '{}'",
                metadata_file.display()
            )
        })?;
    let snapshot_id =
        upload_canister_snapshot_metadata(env, canister_id, None, &metadata, call_sender)
            .await
            .with_context(|| {
                format!(
                    "Failed to upload snapshot metadata to canister {}",
                    canister_id.to_text()
                )
            })?;
    debug!(
        env.get_logger(),
        "Snapshot metadata uploaded to canister {} with Snapshot ID: {}",
        canister_id.to_text(),
        hex::encode(snapshot_id.snapshot_id)
    );

    // TODO: Upload Snapshot data.

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

enum BlobKind {
    WasmModule,
    MainMemory,
    StableMemory,
}

async fn read_blob(
    env: &dyn Environment,
    canister_id: Principal,
    snapshot: &SnapshotId,
    blob_kind: BlobKind,
    length: usize,
    call_sender: &CallSender,
) -> DfxResult<Vec<u8>> {
    const MAX_CHUNK_SIZE: usize = 2_000_000;
    let mut blob: Vec<u8> = vec![0; length];
    let mut offset = 0;
    while offset < length {
        let chunk_size = std::cmp::min(length - offset, MAX_CHUNK_SIZE);
        let kind = match blob_kind {
            BlobKind::WasmModule => SnapshotDataKind::WasmModule {
                offset: offset as u64,
                size: chunk_size as u64,
            },
            BlobKind::MainMemory => SnapshotDataKind::MainMemory {
                offset: offset as u64,
                size: chunk_size as u64,
            },
            BlobKind::StableMemory => SnapshotDataKind::StableMemory {
                offset: offset as u64,
                size: chunk_size as u64,
            },
        };
        let chunk = read_canister_snapshot_data(env, canister_id, &snapshot.0, &kind, call_sender)
            .await
            .with_context(|| {
                format!(
                    "Failed to read data from snapshot {snapshot} from canister {}",
                    canister_id.to_text()
                )
            })?
            .chunk;
        blob[offset..offset + chunk_size].copy_from_slice(&chunk);
        offset += chunk_size;
    }

    Ok(blob)
}
