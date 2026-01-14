use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ic_attributes::CanisterSettings;
use crate::lib::operations::canister::{
    get_canister_status, list_canister_snapshots, update_settings,
};
use crate::lib::operations::canister_migration::{
    MigrationStatus, NNS_MIGRATION_CANISTER_ID, migrate_canister, migration_status,
};
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::subnet::get_subnet_for_canister;
use crate::util::ask_for_consent;
use anyhow::{Context, bail};
use candid::Principal;
use clap::Parser;
use dfx_core::identity::CallSender;
use ic_management_canister_types::CanisterStatusType;
use num_traits::ToPrimitive;
use slog::{debug, error, info};
use std::time::Duration;
use time::{OffsetDateTime, macros::format_description};

/// Migrate a canister ID from one subnet to another.
#[derive(Parser)]
#[command(override_usage = "dfx canister migrate-id [OPTIONS] <CANISTER> --replace <REPLACE>")]
pub struct CanisterMigrateIdOpts {
    /// Specifies the name or id of the canister to migrate.
    canister: String,

    /// Specifies the name or id of the canister to replace.
    #[arg(long)]
    replace: String,

    /// Skips yes/no checks by answering 'yes'. Not recommended outside of CI.
    #[arg(long, short)]
    yes: bool,
}

pub async fn exec(
    env: &dyn Environment,
    opts: CanisterMigrateIdOpts,
    call_sender: &CallSender,
) -> DfxResult {
    fetch_root_key_if_needed(env).await?;

    let log = env.get_logger();
    let agent = env.get_agent();
    let canister_id_store = env.get_canister_id_store()?;

    // Get the canister IDs.
    let migrated_canister = opts.canister.as_str();
    let replaced_canister = opts.replace.as_str();
    let migrated_canister_id = Principal::from_text(migrated_canister)
        .or_else(|_| canister_id_store.get(migrated_canister))?;
    let replaced_canister_id = Principal::from_text(replaced_canister)
        .or_else(|_| canister_id_store.get(replaced_canister))?;

    if migrated_canister_id == replaced_canister_id {
        bail!("The canisters to migrate and replace are identical.");
    }

    if !opts.yes {
        ask_for_consent(
            env,
            &format!(
                "Canister '{migrated_canister}' will be removed from its own subnet. Continue?"
            ),
        )?;
    }

    let migrated_canister_status = get_canister_status(env, migrated_canister_id, call_sender)
        .await
        .with_context(|| format!("Could not retrieve status of canister {migrated_canister}"))?;
    let replaced_canister_status = get_canister_status(env, replaced_canister_id, call_sender)
        .await
        .with_context(|| format!("Could not retrieve status of canister {replaced_canister}"))?;

    // Check that the two canisters are stopped.
    ensure_canister_stopped(migrated_canister_status.status, migrated_canister)?;
    ensure_canister_stopped(replaced_canister_status.status, replaced_canister)?;

    // Check that the canister is ready for migration.
    if !migrated_canister_status.ready_for_migration {
        bail!(
            "Canister '{migrated_canister}' is not ready for migration. Wait a few seconds and try again"
        );
    }

    // Check the cycles balance of migrated canister.
    let cycles = migrated_canister_status
        .cycles
        .0
        .to_u128()
        .context("Unable to parse cycles")?;
    if cycles < 10_000_000_000_000 {
        bail!("Canister '{migrated_canister}' has less than 10T cycles");
    }
    if !opts.yes && cycles > 15_000_000_000_000 {
        ask_for_consent(
            env,
            &format!(
                "Canister '{migrated_canister}' has more than 15T cycles. The extra cycles will get burned during the migration. Continue?"
            ),
        )?;
    }

    // Check that the replaced canister has no snapshots.
    let snapshots = list_canister_snapshots(env, replaced_canister_id, call_sender).await?;
    if !snapshots.is_empty() {
        bail!(
            "The canister '{}' whose canister ID will be replaced has snapshots",
            replaced_canister
        );
    }

    // Check that the two canisters are on different subnets.
    let migrated_canister_subnet = get_subnet_for_canister(agent, migrated_canister_id).await?;
    let replaced_canister_subnet = get_subnet_for_canister(agent, replaced_canister_id).await?;
    if migrated_canister_subnet == replaced_canister_subnet {
        bail!(
            "The canisters '{migrated_canister}' and '{replaced_canister}' are on the same subnet"
        );
    }

    // Add the NNS migration canister as a controller to the migrated canister.
    let mut controllers = migrated_canister_status.settings.controllers.clone();
    if !controllers.contains(&NNS_MIGRATION_CANISTER_ID) {
        controllers.push(NNS_MIGRATION_CANISTER_ID);
        let settings = CanisterSettings {
            controllers: Some(controllers),
            compute_allocation: None,
            memory_allocation: None,
            freezing_threshold: None,
            reserved_cycles_limit: None,
            wasm_memory_limit: None,
            wasm_memory_threshold: None,
            log_visibility: None,
            environment_variables: None,
        };
        update_settings(env, migrated_canister_id, settings, call_sender).await?;
    }

    // Add the NNS migration canister as a controller to the replaced canister.
    let mut controllers = replaced_canister_status.settings.controllers.clone();
    if !controllers.contains(&NNS_MIGRATION_CANISTER_ID) {
        controllers.push(NNS_MIGRATION_CANISTER_ID);
        let settings = CanisterSettings {
            controllers: Some(controllers),
            compute_allocation: None,
            memory_allocation: None,
            freezing_threshold: None,
            reserved_cycles_limit: None,
            wasm_memory_limit: None,
            wasm_memory_threshold: None,
            log_visibility: None,
            environment_variables: None,
        };
        update_settings(env, replaced_canister_id, settings, call_sender).await?;
    }

    // Migrate the from canister to the rename_to canister.
    debug!(
        log,
        "Migrate '{migrated_canister}' replacing '{replaced_canister}'"
    );
    migrate_canister(agent, migrated_canister_id, replaced_canister_id).await?;

    // Wait for migration to complete.
    let spinner = env.new_spinner("Waiting for migration to complete...".into());
    loop {
        match migration_status(agent, migrated_canister_id, replaced_canister_id).await {
            Ok(status) => match status {
                Some(MigrationStatus::InProgress { status }) => {
                    spinner.set_message(format!("Migration in progress: {status}").into());
                }
                Some(MigrationStatus::Succeeded { time }) => {
                    spinner.finish_and_clear();
                    info!(log, "Migration succeeded at {}", format_time(&time));
                    break;
                }
                Some(MigrationStatus::Failed { reason, time }) => {
                    spinner.finish_and_clear();
                    error!(
                        log,
                        "Migration failed at {}: {}",
                        format_time(&time),
                        reason
                    );
                    break;
                }
                None => (),
            },
            Err(e) => {
                spinner.set_message(format!("Could not fetch migration status: {e}").into());
            }
        };

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    canister_id_store.remove(log, replaced_canister)?;

    Ok(())
}

fn ensure_canister_stopped(status: CanisterStatusType, canister: &str) -> DfxResult {
    match status {
        CanisterStatusType::Stopped => Ok(()),
        CanisterStatusType::Running => {
            bail!("Canister {canister} is running. Run 'dfx canister stop' first");
        }
        CanisterStatusType::Stopping => {
            bail!("Canister {canister} is stopping. Wait a few seconds and try again");
        }
    }
}

fn format_time(time: &u64) -> String {
    let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second] UTC");
    OffsetDateTime::from_unix_timestamp_nanos(*time as i128)
        .unwrap()
        .format(&format)
        .unwrap()
}
