use crate::lib::error::DfxResult;
use candid::{CandidType, Principal, Reserved};
use ic_agent::Agent;
use ic_utils::Canister;
use serde::Deserialize;
use std::fmt;

pub const NNS_MIGRATION_CANISTER_ID: Principal =
    Principal::from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x11, 0x01, 0x01]);
const MIGRATE_CANISTER_METHOD: &str = "migrate_canister";
const MIGRATION_STATUS_METHOD: &str = "migration_status";

#[derive(Clone, CandidType, Deserialize)]
pub struct MigrateCanisterArgs {
    pub migrated_canister_id: Principal,
    pub replace_canister_id: Principal,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
enum ValidationError {
    MigrationsDisabled(Reserved),
    RateLimited(Reserved),
    ValidationInProgress { canister: Principal },
    MigrationInProgress { canister: Principal },
    CanisterNotFound { canister: Principal },
    SameSubnet(Reserved),
    CallerNotController { canister: Principal },
    NotController { canister: Principal },
    MigratedNotStopped(Reserved),
    MigratedNotReady(Reserved),
    ReplacedNotStopped(Reserved),
    ReplacedHasSnapshots(Reserved),
    MigratedInsufficientCycles(Reserved),
    CallFailed { reason: String },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::MigrationsDisabled(Reserved) => {
                write!(f, "Canister migrations are disabled at the moment.")
            }
            ValidationError::RateLimited(Reserved) => write!(
                f,
                "Canister migration has been rate-limited. Try again later."
            ),
            ValidationError::ValidationInProgress { canister } => write!(
                f,
                "Validation for canister {canister} is already in progress."
            ),
            ValidationError::MigrationInProgress { canister } => write!(
                f,
                "Canister migration for canister {canister} is already in progress."
            ),
            ValidationError::CanisterNotFound { canister } => {
                write!(f, "The canister {canister} does not exist.")
            }
            ValidationError::SameSubnet(Reserved) => {
                write!(f, "Both canisters are on the same subnet.")
            }
            ValidationError::CallerNotController { canister } => write!(
                f,
                "The canister {canister} is not controlled by the calling identity."
            ),
            ValidationError::NotController { canister } => write!(
                f,
                "The NNS canister sbzkb-zqaaa-aaaaa-aaaiq-cai is not a controller of canister {canister}."
            ),
            ValidationError::MigratedNotStopped(Reserved) => {
                write!(f, "The migrated canister is not stopped.")
            }
            ValidationError::MigratedNotReady(Reserved) => write!(
                f,
                "The migrated canister is not ready for migration. Try again later."
            ),
            ValidationError::ReplacedNotStopped(Reserved) => {
                write!(f, "The replaced canister is not stopped.")
            }
            ValidationError::ReplacedHasSnapshots(Reserved) => {
                write!(f, "The replaced canister has snapshots.")
            }
            ValidationError::MigratedInsufficientCycles(Reserved) => write!(
                f,
                "The migrated canister does not have enough cycles for canister migration. Top up the migrated canister with the required amount of cycles."
            ),
            ValidationError::CallFailed { reason } => {
                write!(f, "Internal IC error: a call failed due to {reason}")
            }
        }
    }
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
    from_canister: Principal,
    to_canister: Principal,
) -> DfxResult {
    let canister = Canister::builder()
        .with_agent(agent)
        .with_canister_id(NNS_MIGRATION_CANISTER_ID)
        .build()?;

    let arg = MigrateCanisterArgs {
        migrated_canister_id: from_canister,
        replace_canister_id: to_canister,
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
    from_canister: Principal,
    to_canister: Principal,
) -> DfxResult<Option<MigrationStatus>> {
    let canister = Canister::builder()
        .with_agent(agent)
        .with_canister_id(NNS_MIGRATION_CANISTER_ID)
        .build()?;

    let arg = MigrateCanisterArgs {
        migrated_canister_id: from_canister,
        replace_canister_id: to_canister,
    };

    let (result,): (Option<MigrationStatus>,) = canister
        .query(MIGRATION_STATUS_METHOD)
        .with_arg(arg)
        .build()
        .await?;

    Ok(result)
}
