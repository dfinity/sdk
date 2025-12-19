use crate::lib::error::DfxResult;
use candid::{CandidType, Principal, Reserved};
use ic_agent::Agent;
use ic_utils::Canister;
use serde::Deserialize;
use std::fmt;
use thiserror::Error;

pub const NNS_MIGRATION_CANISTER_ID: Principal =
    Principal::from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x11, 0x01, 0x01]);
const MIGRATE_CANISTER_METHOD: &str = "migrate_canister";
const MIGRATION_STATUS_METHOD: &str = "migration_status";

#[derive(Clone, CandidType, Deserialize)]
pub struct MigrateCanisterArgs {
    pub migrated_canister_id: Principal,
    pub replace_canister_id: Principal,
}

#[derive(Clone, Debug, Error, CandidType, Deserialize)]
enum ValidationError {
    #[error("Canister migrations are disabled at the moment.")]
    MigrationsDisabled(Reserved),
    #[error("Canister migration has been rate-limited. Try again later.")]
    RateLimited(Reserved),
    #[error("Validation for canister {canister} is already in progress.")]
    ValidationInProgress { canister: Principal },
    #[error("Canister migration for canister {canister} is already in progress.")]
    MigrationInProgress { canister: Principal },
    #[error("The canister {canister} does not exist.")]
    CanisterNotFound { canister: Principal },
    #[error("Both canisters are on the same subnet.")]
    SameSubnet(Reserved),
    #[error("The canister {canister} is not controlled by the calling identity.")]
    CallerNotController { canister: Principal },
    #[error(
        "The NNS canister sbzkb-zqaaa-aaaaa-aaaiq-cai is not a controller of canister {canister}."
    )]
    NotController { canister: Principal },
    #[error("The migrated canister is not stopped.")]
    MigratedNotStopped(Reserved),
    #[error("The migrated canister is not ready for migration. Try again later.")]
    MigratedNotReady(Reserved),
    #[error("The replaced canister is not stopped.")]
    ReplacedNotStopped(Reserved),
    #[error("The replaced canister has snapshots.")]
    ReplacedHasSnapshots(Reserved),
    #[error(
        "The migrated canister does not have enough cycles for canister migration. Top up the migrated canister with the required amount of cycles."
    )]
    MigratedInsufficientCycles(Reserved),
    #[error("Internal IC error: a call failed due to {reason}")]
    CallFailed { reason: String },
}

#[derive(Clone, CandidType, Deserialize, Debug)]
pub enum MigrationStatus {
    InProgress { status: String },
    Failed { reason: String, time: u64 },
    Succeeded { time: u64 },
}

impl fmt::Display for MigrationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MigrationStatus::InProgress { status } => {
                write!(f, "MigrationStatus::InProgress {{ status: {status} }}")
            }
            MigrationStatus::Failed { reason, time } => {
                write!(
                    f,
                    "MigrationStatus::Failed {{ reason: {reason}, time: {time} }}",
                )
            }
            MigrationStatus::Succeeded { time } => {
                write!(f, "MigrationStatus::Succeeded {{ time: {time} }}")
            }
        }
    }
}

pub async fn migrate_canister(
    agent: &Agent,
    migrated_canister: Principal,
    replaced_canister: Principal,
) -> DfxResult {
    let canister = Canister::builder()
        .with_agent(agent)
        .with_canister_id(NNS_MIGRATION_CANISTER_ID)
        .build()?;

    let arg = MigrateCanisterArgs {
        migrated_canister_id: migrated_canister,
        replace_canister_id: replaced_canister,
    };

    let (result,): (Result<(), Option<ValidationError>>,) = canister
        .update(MIGRATE_CANISTER_METHOD)
        .with_arg(arg)
        .build()
        .await?;

    match result {
        Ok(()) => Ok(()),
        Err(None) => Err(anyhow::anyhow!("Validation failed with an unknown error.")),
        Err(Some(err)) => Err(anyhow::anyhow!("Validation failed: {err}")),
    }
}

pub async fn migration_status(
    agent: &Agent,
    migrated_canister: Principal,
    replaced_canister: Principal,
) -> DfxResult<Option<MigrationStatus>> {
    let canister = Canister::builder()
        .with_agent(agent)
        .with_canister_id(NNS_MIGRATION_CANISTER_ID)
        .build()?;

    let arg = MigrateCanisterArgs {
        migrated_canister_id: migrated_canister,
        replace_canister_id: replaced_canister,
    };

    let (result,): (Option<MigrationStatus>,) = canister
        .query(MIGRATION_STATUS_METHOD)
        .with_arg(arg)
        .build()
        .await?;

    Ok(result)
}
