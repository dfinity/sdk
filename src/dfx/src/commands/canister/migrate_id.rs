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
    let source_canister = opts.canister.as_str();
    let target_canister = opts.replace.as_str();
    let source_canister_id = Principal::from_text(source_canister)
        .or_else(|_| canister_id_store.get(source_canister))?;
    let target_canister_id = Principal::from_text(target_canister)
        .or_else(|_| canister_id_store.get(target_canister))?;

    if source_canister_id == target_canister_id {
        bail!("The canisters to migrate and replace are identical.");
    }

    if !opts.yes {
        ask_for_consent(
            env,
            &format!("Canister '{source_canister}' will be removed from its own subnet. Continue?"),
        )?;
    }

    let source_status = get_canister_status(env, source_canister_id, call_sender)
        .await
        .with_context(|| format!("Could not retrieve status of canister {source_canister}"))?;
    let target_status = get_canister_status(env, target_canister_id, call_sender)
        .await
        .with_context(|| format!("Could not retrieve status of canister {target_canister}"))?;

    ensure_canister_stopped(source_status.status, source_canister)?;
    ensure_canister_stopped(target_status.status, target_canister)?;

    // Check the cycles balance of source_canister.
    let cycles = source_status
        .cycles
        .0
        .to_u128()
        .expect("Unable to parse cycles");
    if cycles < 10_000_000_000_000 {
        bail!("Canister '{source_canister}' has less than 10T cycles");
    }
    if !opts.yes && cycles > 15_000_000_000_000 {
        ask_for_consent(
            env,
            &format!(
                "Canister '{source_canister}' has more than 15T cycles. The extra cycles will get burned during the migration. Continue?"
            ),
        )?;
    }

    // Check that the target canister has no snapshots.
    let snapshots = list_canister_snapshots(env, target_canister_id, call_sender).await?;
    if !snapshots.is_empty() {
        bail!(
            "The canister '{}' whose canister ID will be replaced has snapshots",
            target_canister
        );
    }

    // Check that the two canisters are on different subnets.
    let source_subnet = get_subnet_for_canister(agent, source_canister_id).await?;
    let target_subnet = get_subnet_for_canister(agent, target_canister_id).await?;
    if source_subnet == target_subnet {
        bail!("The canisters '{source_canister}' and '{target_canister}' are on the same subnet");
    }

    // Add the NNS migration canister as a controller to the source canister.
    let mut controllers = source_status.settings.controllers.clone();
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
        update_settings(env, source_canister_id, settings, call_sender).await?;
    }

    // Add the NNS migration canister as a controller to the target canister.
    let mut controllers = target_status.settings.controllers.clone();
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
        update_settings(env, target_canister_id, settings, call_sender).await?;
    }

    // Migrate the from canister to the rename_to canister.
    debug!(log, "Migrate '{source_canister}' to '{target_canister}'");
    migrate_canister(agent, source_canister_id, target_canister_id).await?;

    // Wait for migration to complete.
    let spinner = env.new_spinner("Waiting for migration to complete...".into());
    loop {
        match migration_status(agent, source_canister_id, target_canister_id).await {
            Ok(statuses) => match statuses.first() {
                Some(MigrationStatus::InProgress { status }) => {
                    spinner.set_message(format!("Migration in progress: {status}").into());
                }
                Some(MigrationStatus::Succeeded { time }) => {
                    spinner.finish_and_clear();
                    info!(log, "Migration succeeded at {}", format_time(time));
                    break;
                }
                Some(MigrationStatus::Failed { reason, time }) => {
                    spinner.finish_and_clear();
                    error!(log, "Migration failed at {}: {}", format_time(time), reason);
                    break;
                }
                None => (),
            },
            Err(e) => {
                spinner.set_message(format!("Could not fetch migration status: {}", e).into());
            }
        };

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    canister_id_store.remove(log, target_canister)?;

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
