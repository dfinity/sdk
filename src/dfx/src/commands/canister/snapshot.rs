use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use anyhow::{bail, Context};
use clap::{Parser, Subcommand};
use dfx_core::identity::CallSender;
use ic_utils::interfaces::management_canister::CanisterStatus;
use indicatif::HumanBytes;
use itertools::Itertools;
use slog::info;
use time::{macros::format_description, OffsetDateTime};

use crate::lib::{
    environment::Environment,
    error::{DfxError, DfxResult},
    operations::canister::{
        delete_canister_snapshot, get_canister_status, list_canister_snapshots,
        load_canister_snapshot, take_canister_snapshot,
    },
    root_key::fetch_root_key_if_needed,
};

#[derive(Parser)]
pub struct SnapshotOpts {
    #[command(subcommand)]
    subcmd: SnapshotSubcommand,
}

#[derive(Subcommand)]
enum SnapshotSubcommand {
    Create {
        canister: String,
        #[arg(long)]
        replace: Option<SnapshotId>,
        #[arg(long, short)]
        force: bool,
    },
    Load {
        canister: String,
        snapshot: SnapshotId,
        #[arg(long, short)]
        force: bool,
    },
    List {
        canister: String,
    },
    Delete {
        canister: String,
        snapshot: SnapshotId,
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
    let canisters = env.get_canister_id_store()?;
    let logger = env.get_logger();
    match opts.subcmd {
        SnapshotSubcommand::Create {
            canister,
            replace,
            force,
        } => {
            let canister_id = canister.parse().or_else(|_| canisters.get(&canister))?;
            if !force {
                let status = get_canister_status(env, canister_id, call_sender)
                    .await
                    .with_context(|| format!("Could not retrieve status of canister {canister}"))?;
                match status.status {
                    CanisterStatus::Stopped => {}
                    CanisterStatus::Running => bail!("Canister {canister_id} is running and snapshots should not be taken of running canisters. Run `dfx canister stop` first (or override with `--force`)"),
                    CanisterStatus::Stopping => bail!("Canister {canister_id} is stopping but is not yet stopped. Wait a few seconds and try again"),
                }
            }
            let id = take_canister_snapshot(
                env,
                canister_id,
                replace.as_ref().map(|x| &*x.0),
                call_sender,
            )
            .await
            .with_context(|| format!("Failed to take snapshot of canister {canister}"))?;
            info!(
                logger,
                "Created a new snapshot of canister {canister}. Snapshot ID: {}",
                SnapshotId(id.id)
            );
        }
        SnapshotSubcommand::Load {
            canister,
            snapshot,
            force,
        } => {
            let canister_id = canister.parse().or_else(|_| canisters.get(&canister))?;
            if !force {
                let status = get_canister_status(env, canister_id, call_sender)
                    .await
                    .with_context(|| format!("Could not retrieve status of canister {canister}"))?;
                match status.status {
                    CanisterStatus::Stopped => {}
                    CanisterStatus::Running => bail!("Canister {canister} is running and snapshots should not be applied to running canisters. Run `dfx canister stop` first (or override with `--force`)"),
                    CanisterStatus::Stopping => bail!("Canister {canister} is stopping but is not yet stopped. Wait a few seconds and try again"),
                }
            }
            load_canister_snapshot(env, canister_id, &snapshot.0, call_sender)
                .await
                .with_context(|| {
                    format!("Failed to load snapshot {snapshot} in canister {canister}")
                })?;
            info!(logger, "Loaded snapshot {snapshot} in canister {canister}");
        }
        SnapshotSubcommand::Delete { canister, snapshot } => {
            let canister_id = canisters.get(&canister)?;
            delete_canister_snapshot(env, canister_id, &snapshot.0, call_sender)
                .await
                .with_context(|| {
                    format!("Failed to delete snapshot {snapshot} in canister {canister}")
                })?;
            info!(
                logger,
                "Deleted snapshot {snapshot} from canister {canister}"
            );
        }
        SnapshotSubcommand::List { canister } => {
            let canister_id = canister.parse().or_else(|_| canisters.get(&canister))?;
            let snapshots = list_canister_snapshots(env, canister_id, call_sender)
                .await
                .with_context(|| {
                    format!("Failed to retrieve snapshot list from canister {canister}")
                })?;
            if snapshots.is_empty() {
                info!(logger, "No snapshots found in canister {canister}");
            } else {
                let time_fmt =
                    format_description!("[year]-[month]-[day] [hour]:[minute]:[second] UTC");
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
                info!(logger, "{snapshots}");
            }
        }
    }
    Ok(())
}
