use crate::lib::error::DfxResult;
use candid::{CandidType, Principal};
use ic_agent::Agent;
use ic_utils::Canister;
use serde::Deserialize;
use std::fmt;

pub const NNS_MIGRATION_CANISTER_ID: Principal =
    Principal::from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x11, 0x01, 0x01]);
const MIGRATE_CANISTER_METHOD: &str = "migrate_canister";
const MIGRATE_STATUS_METHOD: &str = "migrate_status";

#[derive(CandidType)]
pub struct MigrateCanisterArg {
    pub from_canister: Principal,
    pub to_canister: Principal,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum ValidationError {
    MigrationsDisabled,
    RateLimited,
    MigrationInProgress { canister: Principal },
    CanisterNotFound { canister: Principal },
    SameSubnet,
    CallerNotController { canister: Principal },
    NotController { canister: Principal },
    SourceNotStopped,
    SourceNotReady,
    TargetNotStopped,
    TargetHasSnapshots,
    SourceInsufficientCycles,
    CallFailed { reason: String },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::MigrationsDisabled => write!(f, "MigrationsDisabled"),
            ValidationError::RateLimited => write!(f, "RateLimited"),
            ValidationError::MigrationInProgress { canister } => write!(
                f,
                "ValidationError::MigrationInProgress {{ canister: {canister} }}",
            ),
            ValidationError::CanisterNotFound { canister } => write!(
                f,
                "ValidationError::CanisterNotFound {{ canister: {canister} }}",
            ),
            ValidationError::SameSubnet => write!(f, "SameSubnet"),
            ValidationError::CallerNotController { canister } => write!(
                f,
                "ValidationError::CallerNotController {{ canister: {canister} }}",
            ),
            ValidationError::NotController { canister } => write!(
                f,
                "ValidationError::NotController {{ canister: {canister} }}",
            ),
            ValidationError::SourceNotStopped => write!(f, "SourceNotStopped"),
            ValidationError::SourceNotReady => write!(f, "SourceNotReady"),
            ValidationError::TargetNotStopped => write!(f, "TargetNotStopped"),
            ValidationError::TargetHasSnapshots => write!(f, "TargetHasSnapshots"),
            ValidationError::SourceInsufficientCycles => write!(f, "SourceInsufficientCycles"),
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

    let arg = MigrateCanisterArg {
        from_canister,
        to_canister,
    };

    canister
        .update(MIGRATE_CANISTER_METHOD)
        .with_arg(arg)
        .build()
        .map(|result: (Result<(), ValidationError>,)| (result.0,))
        .await
        .map(|(result,)| result)?
        .map_err(|error| anyhow::anyhow!(error))?;

    Ok(())
}

pub async fn migrate_status(
    agent: &Agent,
    from_canister: Principal,
    to_canister: Principal,
) -> DfxResult<Vec<MigrationStatus>> {
    let canister = Canister::builder()
        .with_agent(agent)
        .with_canister_id(NNS_MIGRATION_CANISTER_ID)
        .build()?;

    let arg = MigrateCanisterArg {
        from_canister,
        to_canister,
    };

    let (result,): (Vec<MigrationStatus>,) = canister
        .query(MIGRATE_STATUS_METHOD)
        .with_arg(arg)
        .build()
        .await?;

    Ok(result)
}
