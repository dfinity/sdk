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
    pub canister_id: Principal,
    pub replace_canister_id: Principal,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum ValidationError {
    MigrationsDisabled(Reserved),
    RateLimited(Reserved),
    ValidationInProgress { canister: Principal },
    MigrationInProgress { canister: Principal },
    CanisterNotFound { canister: Principal },
    SameSubnet(Reserved),
    CallerNotController { canister: Principal },
    NotController { canister: Principal },
    SourceNotStopped(Reserved),
    SourceNotReady(Reserved),
    TargetNotStopped(Reserved),
    TargetHasSnapshots(Reserved),
    SourceInsufficientCycles(Reserved),
    CallFailed { reason: String },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::MigrationsDisabled(Reserved) => write!(f, "MigrationsDisabled"),
            ValidationError::RateLimited(Reserved) => write!(f, "RateLimited"),
            ValidationError::ValidationInProgress { canister } => write!(
                f,
                "ValidationError::ValidationInProgress {{ canister: {canister} }}",
            ),
            ValidationError::MigrationInProgress { canister } => write!(
                f,
                "ValidationError::MigrationInProgress {{ canister: {canister} }}",
            ),
            ValidationError::CanisterNotFound { canister } => write!(
                f,
                "ValidationError::CanisterNotFound {{ canister: {canister} }}",
            ),
            ValidationError::SameSubnet(Reserved) => write!(f, "SameSubnet"),
            ValidationError::CallerNotController { canister } => write!(
                f,
                "ValidationError::CallerNotController {{ canister: {canister} }}",
            ),
            ValidationError::NotController { canister } => write!(
                f,
                "ValidationError::NotController {{ canister: {canister} }}",
            ),
            ValidationError::SourceNotStopped(Reserved) => write!(f, "SourceNotStopped"),
            ValidationError::SourceNotReady(Reserved) => write!(f, "SourceNotReady"),
            ValidationError::TargetNotStopped(Reserved) => write!(f, "TargetNotStopped"),
            ValidationError::TargetHasSnapshots(Reserved) => write!(f, "TargetHasSnapshots"),
            ValidationError::SourceInsufficientCycles(Reserved) => {
                write!(f, "SourceInsufficientCycles")
            }
            ValidationError::CallFailed { reason } => {
                write!(f, "ValidationError::CallFailed {{ reason: {reason} }}")
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
        canister_id: from_canister,
        replace_canister_id: to_canister,
    };

    let _: () = canister
        .update(MIGRATE_CANISTER_METHOD)
        .with_arg(arg)
        .build()
        .await?;

    Ok(())
}

pub async fn migration_status(
    agent: &Agent,
    from_canister: Principal,
    to_canister: Principal,
) -> DfxResult<Vec<MigrationStatus>> {
    let canister = Canister::builder()
        .with_agent(agent)
        .with_canister_id(NNS_MIGRATION_CANISTER_ID)
        .build()?;

    let arg = MigrateCanisterArgs {
        canister_id: from_canister,
        replace_canister_id: to_canister,
    };

    let (result,): (Vec<MigrationStatus>,) = canister
        .query(MIGRATION_STATUS_METHOD)
        .with_arg(arg)
        .build()
        .await?;

    Ok(result)
}
